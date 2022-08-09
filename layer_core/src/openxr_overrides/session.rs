use openxr::sys as xr;

pub(super) unsafe fn get_session_interceptors(_name: &str) -> Option<xr::pfn::VoidFunction> {
    Some(return None)
}
