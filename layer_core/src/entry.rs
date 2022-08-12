use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::wrappers::instance::{InnerInstance, InstanceWrapper, Runtime};
use crate::wrappers::XrHandle;
use crate::{str_from_bytes_until_nul, ToResult};
use openxr::sys::loader_interfaces::*;

use log::{debug, error, info};

use openxr::{sys as xr, Instance};
use openxr::{ExtensionSet, InstanceExtensions, Result};
use parking_lot::Mutex;

pub(crate) unsafe extern "system" fn create_api_layer_instance(
    instance_info: *const xr::InstanceCreateInfo,
    layer_info: *const ApiLayerCreateInfo,
    instance: *mut xr::Instance,
) -> xr::Result {
    std::panic::catch_unwind(|| create_instance(&*instance_info, &*layer_info, &mut *instance))
        .map_or(xr::Result::ERROR_RUNTIME_FAILURE, |res| match res {
            Ok(res) => res,
            Err(res) => res,
        })
}

fn create_instance(
    instance_info: &xr::InstanceCreateInfo,
    layer_info: &ApiLayerCreateInfo,
    instance: &mut xr::Instance,
) -> Result<xr::Result> {
    let next_info = &unsafe { *layer_info.next_info };

    if str_from_bytes_until_nul(&next_info.layer_name[..])? != crate::LAYER_NAME {
        error!(
            "Crate instance failed: Incorrect layer_name `{}`",
            str_from_bytes_until_nul(&next_info.layer_name[..])?
        );
        return Err(xr::Result::ERROR_VALIDATION_FAILURE);
    }

    debug!("Initializing OpenXR Entry");

    let application_name =
        str_from_bytes_until_nul(&(*instance_info).application_info.application_name[..])?;
    //Setup the OpenXR wrapper for the layer bellow us
    let entry = unsafe {
        openxr::Entry::from_get_instance_proc_addr(next_info.next_get_instance_proc_addr)?
    };

    //Initialize the layer bellow us
    let result = unsafe {
        let mut layer_info2 = *layer_info;
        layer_info2.next_info = (*layer_info2.next_info).next;
        (next_info.next_create_api_layer_instance)(instance_info, &layer_info2, instance).result()
    }?;

    let inner = unsafe {
        InnerInstance {
            poison: AtomicBool::new(false),
            core: openxr::raw::Instance::load(&entry, *instance)?,
            exts: InstanceExtensions::load(&entry, *instance, &ExtensionSet::default())?,
        }
    };

    let runtime_name = unsafe {
        let mut instance_properties = xr::InstanceProperties::out(std::ptr::null_mut());
        (inner.core.get_instance_properties)(*instance, instance_properties.as_mut_ptr())
            .result()?;
        let instance_properties = instance_properties.assume_init();

        str_from_bytes_until_nul(&instance_properties.runtime_name[..])?.to_owned()
    };

    let runtime = match runtime_name.deref() {
        "SteamVR/OpenXR" => Runtime::SteamVR,
        "Oculus" => Runtime::Oculus,
        "Windows Mixed Reality Runtime" => Runtime::WMR,
        "Monado(XRT) by Collabora et al" => Runtime::Monado,
        _ => Runtime::Other(runtime_name.to_string()),
    };

    let (suinput_runtime, suinput_instance, suinput_driver) = crate::input::create(
        unsafe { Instance::from_raw(entry, *instance, inner.exts).unwrap() },
        application_name,
    );

    let wrapper = InstanceWrapper {
        handle: *instance,
        inner: Arc::new(inner),
        systems: Default::default(),
        sessions: Default::default(),
        runtime,
        suinput_runtime,
        suinput_instance,
        suinput_driver: Mutex::new(suinput_driver),
    };

    xr::Instance::all_wrappers().insert(*instance, Arc::new(wrapper));

    info!("Instance created with name `{}`", application_name);

    Ok(result)
}
