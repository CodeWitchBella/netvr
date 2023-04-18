#![feature(array_chunks)]
#![feature(const_option)]
#[macro_use]
extern crate lazy_static;

use std::{backtrace::Backtrace, panic};

use implementation::{read_remote_devices, start, tick};
use xr_layer::{
    log::{self, LogPanic},
    pfn,
};

#[macro_use]
mod bincode_abi;
mod data;
mod implementation;
mod instance;
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
    expose read_remote_devices as ReadRemoteDevices taking JustInstance and outputting ReadRemoteDevicesOutput,
    expose tick as Tick taking JustInstance and outputting Nothing,
    expose start as Start taking JustInstance and outputting Nothing,
);
