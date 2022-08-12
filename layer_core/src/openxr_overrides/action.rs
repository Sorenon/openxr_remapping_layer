use openxr::sys as xr;
use thunderdome::Index;

use crate::wrappers::layer_action_set;

pub(super) unsafe fn get_interceptors(name: &str) -> Option<xr::pfn::VoidFunction> {
    use std::mem::transmute;
    use xr::pfn::*;
    Some(match name {
        _ => return None,
    })
}
