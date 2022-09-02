use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use openxr::sys as xr;
use suinput::{
    instance::{ApplicationInfo, ApplicationInstanceCreateInfo},
    SuSession,
};
use thunderdome::Index;

use super::{
    instance::{InnerInstance, InstanceWrapper},
    layer_action::{self, ManySubActions, SingletonAction},
    layer_action_set::{self, LayerActionSet},
    XrHandle, XrWrapper,
};

pub struct SessionWrapper {
    pub handle: xr::Session,
    pub instance: Weak<InstanceWrapper>,
    pub inner_instance: Arc<InnerInstance>,
    pub inner: OnceCell<InnerSession>,
}

impl XrWrapper for SessionWrapper {
    fn inner_instance(&self) -> &Arc<InnerInstance> {
        &self.inner_instance
    }
}

impl XrHandle for xr::Session {
    type Wrapper = SessionWrapper;

    fn all_wrappers<'a>() -> &'a DashMap<Self, Arc<Self::Wrapper>>
    where
        Self: Sized + std::hash::Hash,
    {
        unsafe { super::SESSION_WRAPPERS.get().unwrap() }
    }
}

pub struct InnerSession {
    su_session: SuSession,
    action_sets: HashMap<xr::ActionSet, Arc<LayerActionSet>>,
}

impl SessionWrapper {
    pub fn xr_attach_session_action_sets(
        self: &Arc<Self>,
        action_sets: &[xr::ActionSet],
    ) -> Result<xr::Result, xr::Result> {
        let mut did_set = false;

        self.inner.get_or_try_init(|| {
            did_set = true;
            let instance = self.instance.upgrade().unwrap();

            let all_action_sets = layer_action_set::all();

            let actions_sets = action_sets
                .iter()
                .map(|handle| {
                    let index = Index::from_bits(handle.into_raw())
                        .ok_or(xr::Result::ERROR_HANDLE_INVALID)?;
                    let action_set = all_action_sets
                        .get(index)
                        .ok_or(xr::Result::ERROR_HANDLE_INVALID)?;

                    Ok((*handle, action_set.clone()))
                })
                .collect::<Result<HashMap<_, _>, xr::Result>>()?;

            let application_instance = instance.suinput_instance.create_application_instance(
                &ApplicationInstanceCreateInfo {
                    application_info: &ApplicationInfo {
                        name: crate::str_from_bytes_until_nul(
                            &instance.application_info.application_name[..],
                        )
                        .unwrap(),
                    },
                    sub_name: None,
                    action_sets: &actions_sets
                        .values()
                        .map(|set| &set.su_action_set)
                        .collect::<Vec<_>>()[..],
                    binding_layouts: &[],
                },
            );

            let su_session = application_instance.try_begin_session();

            let driver = instance.suinput_driver.lock();
            driver.bind_session(&su_session, self.handle, &[]);

            Ok(InnerSession {
                su_session,
                action_sets: actions_sets,
            })
        })?;

        if did_set {
            Ok(xr::Result::SUCCESS)
        } else {
            Err(xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED)
        }
    }

    pub fn xr_sync_actions(
        &self,
        active_action_sets: &[xr::ActiveActionSet],
    ) -> Result<xr::Result, xr::Result> {
        let inner = self
            .inner
            .get()
            .ok_or(xr::Result::ERROR_ACTIONSET_NOT_ATTACHED)?;

        let active_sets = active_action_sets
            .iter()
            .map(|active_action_set| {
                assert_eq!(active_action_set.subaction_path, xr::Path::NULL);
                match inner.action_sets.get(&active_action_set.action_set) {
                    Some(layer_action_set) => Some(&layer_action_set.su_action_set),
                    None => None,
                }
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(xr::Result::ERROR_ACTIONSET_NOT_ATTACHED)?;

        inner.su_session.sync(&active_sets[..]);

        Ok(xr::Result::SUCCESS)
    }

    pub fn xr_get_action_state_boolean(
        self: &Arc<Self>,
        action: xr::Action,
        sub_action_path: xr::Path,
        out: &mut xr::ActionStateBoolean,
    ) -> Result<xr::Result, xr::Result> {
        let inner = self
            .inner
            .get()
            .ok_or(xr::Result::ERROR_ACTIONSET_NOT_ATTACHED)?;

        let layer_actions = layer_action::all();
        let action = layer_action::get(&layer_actions, action)?;

        match &action.sub_actions {
            layer_action::SubActions::None(action) => {
                if sub_action_path != xr::Path::NULL {
                    return Err(xr::Result::ERROR_PATH_INVALID);
                }

                match action {
                    SingletonAction::Boolean(action) => {
                        let action_state = inner
                            .su_session
                            .get_action_state(action)
                            .expect("TODO handle error");

                        out.is_active = true.into();
                        out.current_state = action_state.into();
                        out.changed_since_last_sync = true.into(); //TODO
                        out.last_change_time = xr::Time::from_nanos(0); //TODO
                    }
                    _ => return Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH),
                }
            }
            layer_action::SubActions::Some(sub_actions) => {
                let actions = match sub_actions {
                    ManySubActions::Boolean(actions) => actions,
                    _ => return Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH),
                };

                for (path, action) in actions {
                    if *path != sub_action_path {
                        continue;
                    }

                    let action_state = inner
                        .su_session
                        .get_action_state(action)
                        .expect("TODO handle error");

                    out.is_active = true.into();
                    out.current_state = action_state.into();
                    out.changed_since_last_sync = true.into(); //TODO
                    out.last_change_time = xr::Time::from_nanos(0); //TODO
                    
                    return Ok(xr::Result::SUCCESS);
                }

                return Err(xr::Result::ERROR_PATH_INVALID);
            }
        }

        Ok(xr::Result::SUCCESS)
    }
}
