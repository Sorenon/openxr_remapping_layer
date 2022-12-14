use std::{ffi::CStr, fs::File, path::Path};

use log::{debug, error, info};
use once_cell::sync::OnceCell;
use openxr::sys as xr;
use simplelog::*;
use xr::loader_interfaces::*;

static LOGGER_LOADED: OnceCell<()> = OnceCell::new();

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn xrNegotiateLoaderApiLayerInterface(
    negotiate_info: *const XrNegotiateLoaderInfo,
    layer_name: *const i8,
    layer_request: *mut XrNegotiateApiLayerRequest,
) -> xr::Result {
    LOGGER_LOADED.get_or_init(|| {
        let workspace_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create(workspace_path.join("log.txt")).unwrap(),
            ),
        ])
        .unwrap();
    });

    info!("Initializing layer");

    if CStr::from_ptr(layer_name).to_string_lossy() != layer_core::LAYER_NAME {
        error!(
            "Layer negotiation failed: Incorrect layer_name `{}`",
            CStr::from_ptr(layer_name).to_string_lossy()
        );
        xr::Result::ERROR_INITIALIZATION_FAILED
    } else if (*negotiate_info).min_interface_version > CURRENT_LOADER_API_LAYER_VERSION
        || (*negotiate_info).max_interface_version < CURRENT_LOADER_API_LAYER_VERSION
        || (*negotiate_info).min_api_version > xr::CURRENT_API_VERSION
        || (*negotiate_info).max_api_version < xr::CURRENT_API_VERSION
    {
        error!(
            "Layer negotiation failed: Incompatible negotiate info {:#?}",
            (*negotiate_info)
        );
        xr::Result::ERROR_INITIALIZATION_FAILED
    } else {
        let (get_instance_proc_addr, create_api_layer_instance) = layer_core::initialize();

        (*layer_request).layer_interface_version = CURRENT_LOADER_API_LAYER_VERSION;
        (*layer_request).layer_api_version = xr::CURRENT_API_VERSION;
        (*layer_request).get_instance_proc_addr = Some(get_instance_proc_addr);
        (*layer_request).create_api_layer_instance = Some(create_api_layer_instance);

        debug!("Negotiation complete");

        xr::Result::SUCCESS
    }
}
