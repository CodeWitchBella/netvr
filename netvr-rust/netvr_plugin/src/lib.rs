#[macro_use]
extern crate lazy_static;

use std::{backtrace::Backtrace, panic};

use implementation::{
    get_server_address, grab, init_remote_objects, object_set_pose, read_remote_devices,
    read_remote_objects, release, start,
};
use xr_layer::{
    log::{self, LogPanic},
    pfn,
};

#[macro_use]
mod bincode_abi;
mod config;
mod implementation;
mod instance;
mod local_configuration;
mod net_client;
mod overrides;
mod xr_wrap;

/// this gets called from unity to give us option to override basically any
/// openxr function it only calls into loader and is what injects our safe rust
/// implementation
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<pfn::GetInstanceProcAddr>,
    manual_unhook: bool,
) -> Option<pfn::GetInstanceProcAddr> {
    func_in.map(|func| overrides::init(func, manual_unhook))
}

/// Deinitializes netvr. Needs to be called only if manual_unhook was specified
/// for hook function. Must be called after everything openxr-related is already
/// finished, but possibly before xrDestroyInstance.
#[no_mangle]
pub extern "C" fn netvr_unhook() {
    overrides::deinit()
}

/// Sets logger function to be used by the layer. This allows you to surface
/// netvr logs to for example unity editor.
#[no_mangle]
pub extern "C" fn netvr_set_logger(func: log::LoggerFn) {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::capture();
        let mut message = match panic_info.location() {
            Some(location) => format!(
                "panic occurred in file '{}' at line {}",
                location.file(),
                location.line(),
            ),
            None => "panic occurred but can't get location information...".to_string(),
        };

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            message = format!("{message}: {s:?}");
        }
        message = format!("{message}\n{panic_info:?}\n{backtrace:?}");
        LogPanic::string(message);
    }));

    log::set_logger(func)
}

pub use bincode_abi::netvr_cleanup;

bincode_expose!(
    expose read_remote_devices as ReadRemoteDevices taking InstanceAndSession and outputting ReadRemoteDevicesOutput,
    expose start as Start taking StartInput and outputting Nothing,
    expose read_remote_objects as ReadRemoteObjects taking InstanceAndSession and outputting Snapshot,
    expose init_remote_objects as InitRemoteObjects taking InitRemoteObjectsInput and outputting Nothing,
    expose grab as Grab taking GrabInput and outputting Nothing,
    expose release as Release taking GrabInput and outputting Nothing,
    expose object_set_pose as ObjectSetPose taking SetPoseInput and outputting Nothing,
    expose get_server_address as GetServerAddress taking InstanceAndSession and outputting OnlyString,
);
