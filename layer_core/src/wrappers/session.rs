use std::sync::{Arc, Weak};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use openxr::sys as xr;
use suinput::SuSession;

use super::{
    instance::{InnerInstance, InstanceWrapper},
    XrHandle, XrWrapper,
};

pub struct SessionWrapper {
    pub handle: xr::Session,
    pub instance: Weak<InstanceWrapper>,
    pub inner: Arc<InnerInstance>,
    pub suinput_session: OnceCell<SuSession>,
}

impl XrWrapper for SessionWrapper {
    fn inner_instance(&self) -> &Arc<InnerInstance> {
        &self.inner
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

impl SessionWrapper {
    pub fn xr_attach_session_action_sets(
        self: &Arc<Self>,
        attach_info: &xr::SessionActionSetsAttachInfo,
    ) -> Result<xr::Result, xr::Result> {
        let instance = self.instance.upgrade().unwrap();
        let driver = instance.suinput_driver.lock();

        driver.add_session(self.handle);

        Ok(xr::Result::SUCCESS)
    }
}
