use backtrace::Backtrace;
use std::panic;

use implementation::ImplementationInstance;
use xr_layer::{
    loader::XrLayerLoader,
    log::{self, LogPanic},
    openxr::sys::{self, pfn},
};

mod implementation;

type Loader = XrLayerLoader<ImplementationInstance>;
// this gets called from unity to give us option to override basically any openxr function
// it only calls into loader and is what injects our safe rust implementation
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<pfn::GetInstanceProcAddr>,
    automatic_destroy: bool,
) -> Option<pfn::GetInstanceProcAddr> {
    Loader::hook_get_instance_proc_addr(func_in, automatic_destroy)
}

#[no_mangle]
pub extern "C" fn netvr_manual_destroy_instance(instance_handle: sys::Instance) {
    Loader::netvr_manual_destroy_instance(instance_handle);
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
