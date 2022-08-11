use openxr::{Entry, Instance};
use openxr_driver::OpenXRDriver;
use suinput::{SuInputRuntime, SuInstance};

pub fn create(instance: Instance, name: &str) -> (SuInputRuntime, SuInstance, OpenXRDriver) {
    let runtime = suinput::load_runtime();
    let driver = openxr_driver::OpenXRDriver::new(instance);

    let instance = runtime.create_instance(name);

    (runtime, instance, driver)
}
