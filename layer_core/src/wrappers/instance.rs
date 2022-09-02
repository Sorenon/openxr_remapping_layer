use std::{sync::{atomic::AtomicBool, Arc}, collections::HashMap};

use dashmap::DashMap;
use log::{debug, info};
use once_cell::sync::OnceCell;
use openxr::{sys as xr, Path};
use openxr_driver::OpenXRDriver;
use parking_lot::Mutex;
use suinput::{instance::SuInstance, SuInputRuntime, SuPath, SuBindingLayout};

use crate::{str_from_bytes_until_nul, ToResult, input::suggested_bindings::SuggestedBindings};

use super::{
    layer_action_set::{self, LayerActionSet},
    session::SessionWrapper,
    XrHandle, XrWrapper,
};

pub struct InstanceWrapper {
    pub handle: xr::Instance,
    pub application_info: xr::ApplicationInfo,
    pub inner: Arc<InnerInstance>,
    pub systems: DashMap<xr::SystemId, SystemMeta>,
    pub sessions: DashMap<xr::Session, Arc<SessionWrapper>>,
    pub runtime: Runtime,
    pub suinput_runtime: SuInputRuntime,
    pub suinput_instance: SuInstance,
    pub suinput_driver: Mutex<OpenXRDriver>,
    pub suggested_bindings: Mutex<HashMap<SuPath, SuggestedBindings>>,
}

pub struct InnerInstance {
    pub poison: AtomicBool,
    pub instance: openxr::sys::Instance,
    pub core: openxr::raw::Instance,
    pub exts: openxr::InstanceExtensions,
}

pub struct SystemMeta {
    pub form_factor: xr::FormFactor,
}

pub enum Runtime {
    SteamVR,
    Oculus,
    WMR,
    Monado,
    Other(String),
}

impl XrWrapper for InstanceWrapper {
    fn inner_instance(&self) -> &Arc<InnerInstance> {
        &self.inner
    }
}

impl XrHandle for xr::Instance {
    type Wrapper = InstanceWrapper;

    fn all_wrappers<'a>() -> &'a DashMap<Self, Arc<Self::Wrapper>>
    where
        Self: Sized + std::hash::Hash,
    {
        unsafe { super::INSTANCE_WRAPPERS.get().unwrap() }
    }
}

impl InnerInstance {
    pub fn path_to_string(&self, path: xr::Path) -> Result<String, xr::Result> {
        crate::ffi_helpers::get_str(|input, output, buf| unsafe {
            (self.core.path_to_string)(self.instance, path, input, output, buf)
        })
    }
}

impl InstanceWrapper {
    pub fn xr_get_system(
        self: &Arc<Self>,
        get_info: &xr::SystemGetInfo,
        system_id: &mut xr::SystemId,
    ) -> Result<xr::Result, xr::Result> {
        let success =
            unsafe { (self.inner.core.get_system)(self.handle, get_info, system_id) }.result()?;

        self.systems.insert(
            *system_id,
            SystemMeta {
                form_factor: get_info.form_factor,
            },
        );

        debug!(
            "Get system called: form_factor={:?}, id={}",
            get_info.form_factor,
            system_id.into_raw()
        );

        Ok(success)
    }

    pub fn xr_create_session(
        self: &Arc<Self>,
        create_info: &xr::SessionCreateInfo,
        session: &mut xr::Session,
    ) -> Result<xr::Result, xr::Result> {
        unsafe { (self.inner.core.create_session)(self.handle, create_info, session) }.result()?;

        let session_wrapper = Arc::new(SessionWrapper {
            handle: *session,
            instance: Arc::downgrade(self),
            inner_instance: self.inner.clone(),
            inner: OnceCell::new(),
        });

        *session = session_wrapper.handle;
        xr::Session::all_wrappers().insert(*session, session_wrapper.clone());
        self.sessions.insert(*session, session_wrapper);
        info!("Session created: {:?}", *session);

        Ok(xr::Result::SUCCESS)
    }

    pub fn xr_create_action_set(
        self: &Arc<Self>,
        create_info: &xr::ActionSetCreateInfo,
        handle_out: &mut xr::ActionSet,
    ) -> Result<xr::Result, xr::Result> {
        let name = str_from_bytes_until_nul(&create_info.action_set_name[..])?;
        let su_action_set = self
            .suinput_instance
            .create_action_set(name, create_info.priority);

        let handle = layer_action_set::all_mut().insert(Arc::new(LayerActionSet {
            instance: Arc::downgrade(self),
            inner: self.inner.clone(),
            su_action_set,
        }));

        *handle_out = xr::ActionSet::from_raw(handle.to_bits());

        Ok(xr::Result::SUCCESS)
    }

    pub fn xr_suggest_interaction_profile_bindings(
        self: &Arc<Self>,
        interaction_profile: Path,
        suggested_bindings: &[xr::ActionSuggestedBinding],
    ) -> Result<xr::Result, xr::Result> {
        let interaction_profile_string = self.inner.path_to_string(interaction_profile)?;
        let su_interaction_profile_path = self
            .suinput_instance
            .get_path(&interaction_profile_string)
            .map_err(|_| xr::Result::ERROR_PATH_UNSUPPORTED)?;

        

            

        Ok(xr::Result::SUCCESS)
    }
}
