#![feature(cstr_from_bytes_until_nul)]

mod entry;
mod input;
pub mod openxr_overrides;
pub mod wrappers;

use std::ffi::CStr;

use openxr::sys as xr;

pub const LAYER_NAME: &str = "XR_APILAYER_SORENON_suinput_layer";

pub fn initialize() -> (
    xr::pfn::GetInstanceProcAddr,
    xr::loader_interfaces::FnCreateApiLayerInstance,
) {
    wrappers::initialize();
    (
        openxr_overrides::get_instance_proc_addr,
        entry::create_api_layer_instance,
    )
}

pub trait ToResult {
    fn result(self) -> Result<Self, Self>
    where
        Self: Sized + Copy,
    {
        ToResult::result2(self, self)
    }

    fn result2<T>(self, ok: T) -> Result<T, Self>
    where
        Self: Sized + Copy;
}

impl ToResult for xr::Result {
    fn result2<T>(self, ok: T) -> Result<T, Self> {
        if self.into_raw() >= 0 {
            Ok(ok)
        } else {
            Err(self)
        }
    }
}

pub fn str_from_bytes_until_nul(bytes: &[i8]) -> Result<&str, xr::Result> {
    let bytes = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };

    let cstr =
        CStr::from_bytes_until_nul(bytes).map_err(|_| xr::Result::ERROR_VALIDATION_FAILURE)?;

    cstr.to_str()
        .map_err(|_| xr::Result::ERROR_VALIDATION_FAILURE)
}
