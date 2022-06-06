use implementation::ImplementationInstance;
use loader::XrLayerLoader;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
mod implementation;
mod loader;
mod log;
mod xr_functions;
mod xr_structures;

type Loader = XrLayerLoader<ImplementationInstance>;
// this gets called from unity to give us option to override basically any openxr function
// it only calls into loader and is what injects our safe rust implementation
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<openxr_sys::pfn::GetInstanceProcAddr>,
    automatic_destroy: bool,
) -> Option<openxr_sys::pfn::GetInstanceProcAddr> {
    return Loader::hook_get_instance_proc_addr(func_in, automatic_destroy);
}

#[no_mangle]
pub extern "C" fn netvr_manual_destroy_instance(instance_handle: openxr_sys::Instance) {
    Loader::netvr_manual_destroy_instance(instance_handle);
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(func: log::LoggerFn) {
    log::set_logger(func)
}
