use openxr::sys as xr;

use crate::wrappers::XrHandle;

pub(super) unsafe fn get_interceptors(name: &str) -> Option<xr::pfn::VoidFunction> {
    use std::mem::transmute;
    use xr::pfn::*;
    Some(match name {
        "xrAttachSessionActionSets" => {
            transmute(xr_attach_session_action_sets as AttachSessionActionSets)
        }
        "xrSyncAction" => transmute(sync_actions as SyncActions),
        _ => return None,
    })
}

unsafe extern "system" fn xr_attach_session_action_sets(
    session: xr::Session,
    attach_info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    let attach_info = &*attach_info;
    session.run(|session| {
        session.xr_attach_session_action_sets(std::slice::from_raw_parts(
            attach_info.action_sets,
            attach_info.count_action_sets as usize,
        ))
    })
}

unsafe extern "system" fn sync_actions(
    session: xr::Session,
    sync_info: *const xr::ActionsSyncInfo,
) -> xr::Result {
    let sync_info = &*sync_info;
    session.run(|session| {
        session.xr_sync_actions(std::slice::from_raw_parts(
            sync_info.active_action_sets,
            sync_info.count_active_action_sets as usize,
        ))
    })
}
