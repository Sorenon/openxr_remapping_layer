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

// Copied from OpenXR-rs
mod ffi_helpers {
    fn cvt(x: openxr::sys::Result) -> openxr::Result<openxr::sys::Result> {
        if x.into_raw() >= 0 {
            Ok(x)
        } else {
            Err(x)
        }
    }
    
    fn place_cstr(out: &mut [std::os::raw::c_char], s: &str) {
        if s.len() + 1 > out.len() {
            panic!(
                "string requires {} > {} bytes (including trailing null)",
                s.len(),
                out.len()
            );
        }
        for (i, o) in s.bytes().zip(out.iter_mut()) {
            *o = i as std::os::raw::c_char;
        }
        out[s.len()] = 0;
    }
    
    unsafe fn fixed_str(x: &[std::os::raw::c_char]) -> &str {
        std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(x.as_ptr()).to_bytes())
    }
    
    /// Includes null for convenience of comparison with C string constants
    fn fixed_str_bytes(x: &[std::os::raw::c_char]) -> &[u8] {
        let end = x.iter().position(|&x| x == 0).unwrap();
        unsafe { std::mem::transmute(&x[..=end]) }
    }
    
    pub fn get_str(mut getter: impl FnMut(u32, &mut u32, *mut std::os::raw::c_char) -> openxr::sys::Result) -> openxr::Result<String> {
        let mut bytes = get_arr(|x, y, z| getter(x, y, z as _))?;
        // Truncate at first null byte
        let first_nt = bytes
            .iter()
            .rposition(|&x| x != 0)
            .map(|x| x + 1)
            .unwrap_or(0);
        bytes.truncate(first_nt);
    
        unsafe { Ok(String::from_utf8_unchecked(bytes)) }
    }
    
    fn get_arr<T: Copy>(
        mut getter: impl FnMut(u32, &mut u32, *mut T) -> openxr::sys::Result,
    ) -> openxr::Result<Vec<T>> {
        let mut output = 0;
        cvt(getter(0, &mut output, std::ptr::null_mut()))?;
        let mut buffer = Vec::with_capacity(output as usize);
        loop {
            match cvt(getter(
                buffer.capacity() as u32,
                &mut output,
                buffer.as_mut_ptr() as _,
            )) {
                Ok(_) => {
                    unsafe {
                        buffer.set_len(output as usize);
                    }
                    return Ok(buffer);
                }
                Err(openxr::sys::Result::ERROR_SIZE_INSUFFICIENT) => {
                    buffer.reserve(output as usize - buffer.capacity());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    
    fn get_arr_init<T: Copy>(
        init: T,
        mut getter: impl FnMut(u32, &mut u32, *mut T) -> openxr::sys::Result,
    ) -> openxr::Result<Vec<T>> {
        let mut output = 0;
        cvt(getter(0, &mut output, std::ptr::null_mut()))?;
        let mut buffer = vec![init; output as usize];
        loop {
            match cvt(getter(output, &mut output, buffer.as_mut_ptr() as _)) {
                Ok(_) => {
                    buffer.truncate(output as usize);
                    return Ok(buffer);
                }
                Err(openxr::sys::Result::ERROR_SIZE_INSUFFICIENT) => {
                    buffer.resize(output as usize, init);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }    
}