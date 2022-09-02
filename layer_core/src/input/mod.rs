use openxr::Instance;
use openxr_driver::OpenXRDriver;
use suinput::{instance::SuInstance, SuInputRuntime};

pub mod suggested_bindings;

pub fn create(instance: Instance) -> (SuInputRuntime, SuInstance, OpenXRDriver) {
    let runtime = suinput::load_runtime();
    let driver = openxr_driver::OpenXRDriver::new(instance);

    //TODO
    let instance = runtime.create_instance(None);

    (runtime, instance, driver)
}
