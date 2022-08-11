use openxr::sys as xr;

use crate::wrappers::XrHandle;

pub(super) unsafe fn get_interceptors(name: &str) -> Option<xr::pfn::VoidFunction> {
    use std::mem::transmute;
    use xr::pfn::*;
    Some(match name {
        "xrAttachSessionActionSets" => {
            transmute(xr_attach_session_action_sets as AttachSessionActionSets)
        }
        _ => return None,
    })
}

unsafe extern "system" fn xr_attach_session_action_sets(
    session: xr::Session,
    attach_info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    session.run(|session| session.xr_attach_session_action_sets(&*attach_info))
}
