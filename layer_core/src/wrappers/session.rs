use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use openxr::sys as xr;
use suinput::SuSession;
use thunderdome::Index;

use super::{
    instance::{InnerInstance, InstanceWrapper},
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
            let driver = instance.suinput_driver.lock();
            driver.add_session(self.handle);

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

            Ok(InnerSession {
                su_session: instance.suinput_instance.create_session(
                    &actions_sets
                        .values()
                        .map(|set| &set.su_action_set)
                        .collect::<Vec<_>>()[..],
                ),
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
                active_action_set.action_set
            })
            .collect::<Vec<_>>();

        Ok(xr::Result::SUCCESS)
    }
}
