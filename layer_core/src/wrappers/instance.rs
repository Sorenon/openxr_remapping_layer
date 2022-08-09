use std::sync::{atomic::AtomicBool, Arc};

use dashmap::DashMap;
use log::{debug, info};
use openxr::sys as xr;

use crate::ToResult;

use super::{session::SessionWrapper, XrHandle, XrWrapper};

pub struct InstanceWrapper {
    pub handle: xr::Instance,
    pub inner: Arc<InnerInstance>,
    pub systems: DashMap<xr::SystemId, SystemMeta>,
    pub sessions: DashMap<xr::Session, Arc<SessionWrapper>>,
    pub runtime: Runtime,
}

pub struct InnerInstance {
    pub poison: AtomicBool,
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
            inner: self.inner.clone(),
        });

        *session = session_wrapper.handle;
        xr::Session::all_wrappers().insert(*session, session_wrapper.clone());
        self.sessions.insert(*session, session_wrapper);
        info!("Session created: {:?}", *session);

        Ok(xr::Result::SUCCESS)
    }
}
