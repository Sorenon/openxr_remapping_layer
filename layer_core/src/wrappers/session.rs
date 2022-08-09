use std::sync::{Arc, Weak};

use dashmap::DashMap;
use openxr::sys as xr;

use super::{
    instance::{InnerInstance, InstanceWrapper},
    XrHandle, XrWrapper,
};

pub struct SessionWrapper {
    pub handle: xr::Session,
    pub instance: Weak<InstanceWrapper>,
    pub inner: Arc<InnerInstance>,
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
