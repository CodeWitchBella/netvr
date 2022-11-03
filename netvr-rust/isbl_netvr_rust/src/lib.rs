#[macro_use]
extern crate lazy_static;

use backtrace::Backtrace;
use implementation::tick;
use overrides::with_instance;
use std::panic;
use xr_wrap::xr_wrap_trace;

//use implementation::ImplementationInstance;
use xr_layer::{
    log::{self, LogPanic},
    pfn, sys,
};

mod implementation;
mod instance;
mod overrides;
mod xr_wrap;

// this gets called from unity to give us option to override basically any openxr function
// it only calls into loader and is what injects our safe rust implementation
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<pfn::GetInstanceProcAddr>,
    manual_unhook: bool,
) -> Option<pfn::GetInstanceProcAddr> {
    func_in.map(|func| overrides::init(func, manual_unhook))
}

#[no_mangle]
pub extern "C" fn netvr_unhook() {
    overrides::deinit()
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(func: log::LoggerFn) {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
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

#[no_mangle]
pub extern "C" fn netvr_tick(instance_handle: sys::Instance) {
    xr_wrap_trace("netvr_tick", || {
        with_instance(instance_handle, |instance| {
            tick(instance_handle, instance);
            Ok(())
        })
    });
}
