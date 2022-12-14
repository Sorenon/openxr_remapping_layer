use openxr::sys::{self as xr, pfn};

use crate::wrappers::XrHandle;

pub(super) unsafe fn get_interceptors(name: &str) -> Option<pfn::VoidFunction> {
    use std::mem::transmute;
    use xr::pfn::*;
    Some(match name {
        "xrGetSystem" => transmute(xr_get_system as GetSystem),
        "xrCreateSession" => transmute(xr_create_session as CreateSession),
        "xrCreateActionSet" => transmute(xr_create_action_set as CreateActionSet),
        "xrSuggestInteractionProfileBindings" => {
            transmute(xr_suggest_interaction_profile_bindings as SuggestInteractionProfileBindings)
        }
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

unsafe extern "system" fn xr_create_action_set(
    instance: xr::Instance,
    create_info: *const xr::ActionSetCreateInfo,
    handle: *mut xr::ActionSet,
) -> xr::Result {
    instance.run(|instance| instance.xr_create_action_set(&*create_info, &mut *handle))
}

unsafe extern "system" fn xr_suggest_interaction_profile_bindings(
    instance: xr::Instance,
    suggested_bindings: *const xr::InteractionProfileSuggestedBinding,
) -> xr::Result {
    let suggested_bindings = &*suggested_bindings;

    instance.run(|instance| {
        instance.xr_suggest_interaction_profile_bindings(
            suggested_bindings.interaction_profile,
            std::slice::from_raw_parts(
                suggested_bindings.suggested_bindings,
                suggested_bindings.count_suggested_bindings as usize,
            ),
        )
    })
}
