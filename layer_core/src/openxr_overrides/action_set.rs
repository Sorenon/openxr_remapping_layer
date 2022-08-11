use openxr::sys as xr;
use thunderdome::Index;

use crate::wrappers::layer_action_set;

pub(super) unsafe fn get_interceptors(name: &str) -> Option<xr::pfn::VoidFunction> {
    use std::mem::transmute;
    use xr::pfn::*;
    Some(match name {
        "xrCreateAction" => transmute(xr_create_action as CreateAction),
        _ => return None,
    })
}

unsafe extern "system" fn xr_create_action(
    action_set: xr::ActionSet,
    create_info: *const xr::ActionCreateInfo,
    action: *mut xr::Action,
) -> xr::Result {
    let index = match Index::from_bits(action_set.into_raw()) {
        Some(index) => index,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    let lock = layer_action_set::all();
    let wrapper = match lock.get(index) {
        Some(wrapper_ref) => wrapper_ref,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };
    match wrapper.xr_create_action(&*create_info, &mut *action) {
        Ok(res) => res,
        Err(res) => res,
    }
}
