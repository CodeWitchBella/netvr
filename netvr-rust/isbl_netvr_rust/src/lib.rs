#[macro_use]
extern crate lazy_static;

use implementation::tick;
use overrides::with_layer;
use std::panic;
use std::{backtrace::Backtrace, ffi::c_void};
use tracing::{span, Level};

//use implementation::ImplementationInstance;
use xr_layer::{
    log::{self, LogPanic},
    pfn, sys,
};

mod implementation;
mod instance;
mod overrides;
mod xr_wrap;

/// this gets called from unity to give us option to override basically any openxr function
/// it only calls into loader and is what injects our safe rust implementation
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

/// Should be called periodically. Used to do network upkeep separate from openxr's
/// rendering loop.
#[no_mangle]
pub extern "C" fn netvr_tick(instance_handle: sys::Instance) {
    xr_wrap::xr_wrap(|| {
        with_layer(instance_handle, |instance| {
            let _span = span!(Level::TRACE, "netvr_tick").entered();
            tick(instance)
        })
    });
}

#[no_mangle]
pub extern "C" fn netvr_read_remote_device_data(
    instance_handle: sys::Instance,
    length: u32,
    data: c_void,
) {
}

#[no_mangle]
pub extern "C" fn netvr_read_remote_device_count(instance_handle: sys::Instance) -> u32 {
    0
}
