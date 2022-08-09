pub mod instance;
pub mod session;

use std::{
    hash::Hash,
    panic::{RefUnwindSafe, UnwindSafe},
    sync::{atomic::Ordering, Arc},
};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use openxr::sys as xr;

use self::instance::InnerInstance;

static mut INSTANCE_WRAPPERS: OnceCell<DashMap<xr::Instance, Arc<instance::InstanceWrapper>>> =
    OnceCell::new();
static mut SESSION_WRAPPERS: OnceCell<DashMap<xr::Session, Arc<session::SessionWrapper>>> =
    OnceCell::new();

pub(crate) fn initialize() {
    unsafe {
        INSTANCE_WRAPPERS.get_or_init(DashMap::new);
        SESSION_WRAPPERS.get_or_init(DashMap::new);
    }
}

pub trait XrWrapper {
    fn inner_instance(&self) -> &Arc<InnerInstance>;
}

pub trait XrHandle {
    type Wrapper: XrWrapper;

    fn all_wrappers<'a>() -> &'a DashMap<Self, Arc<Self::Wrapper>>
    where
        Self: Sized + Hash;

    fn run<F>(self, f: F) -> xr::Result
    where
        Self: Sized + Copy + Hash + Eq + RefUnwindSafe,
        F: FnOnce(&Arc<Self::Wrapper>) -> openxr::Result<xr::Result> + UnwindSafe,
    {
        match std::panic::catch_unwind(|| {
            let wrapper = match Self::all_wrappers().get(&self) {
                Some(wrapper_ref) => wrapper_ref,
                None => return xr::Result::ERROR_HANDLE_INVALID,
            };
            if wrapper.inner_instance().poison.load(Ordering::Relaxed) {
                xr::Result::ERROR_INSTANCE_LOST
            } else {
                match f(wrapper.value()) {
                    Ok(res) => res,
                    Err(res) => res,
                }
            }
        }) {
            Ok(res) => res,
            Err(_) => {
                if let Some(wrapper) = Self::all_wrappers().get(&self) {
                    wrapper
                        .inner_instance()
                        .poison
                        .store(true, Ordering::Relaxed);
                }
                xr::Result::ERROR_INSTANCE_LOST
            }
        }
    }
}
