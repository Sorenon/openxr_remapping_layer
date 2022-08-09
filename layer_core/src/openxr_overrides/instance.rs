use openxr::sys::{self as xr, pfn};

use crate::wrappers::XrHandle;

pub(super) unsafe fn get_instance_interceptors(name: &str) -> Option<pfn::VoidFunction> {
    use std::mem::transmute;
    use xr::pfn::*;
    Some(match name {
        "xrGetSystem" => transmute(xr_get_system as GetSystem),
        "xrCreateSession" => transmute(xr_create_session as CreateSession),
        _ => return None,
    })
}

unsafe extern "system" fn xr_get_system(
    instance: xr::Instance,
    get_info: *const xr::SystemGetInfo,
    system_id: *mut xr::SystemId,
) -> xr::Result {
    instance.run(|instance| instance.xr_get_system(&*get_info, &mut *system_id))
}

unsafe extern "system" fn xr_create_session(
    instance: xr::Instance,
    create_info: *const xr::SessionCreateInfo,
    session: *mut xr::Session,
) -> xr::Result {
    instance.run(|instance| instance.xr_create_session(&*create_info, &mut *session))
}
