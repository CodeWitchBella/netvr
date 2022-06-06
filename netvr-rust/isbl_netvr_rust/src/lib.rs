use implementation::ImplementationInstance;
use xr_layer::{loader::XrLayerLoader, log, pfn, XrInstance};

mod implementation;

type Loader = XrLayerLoader<ImplementationInstance>;
// this gets called from unity to give us option to override basically any openxr function
// it only calls into loader and is what injects our safe rust implementation
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<pfn::GetInstanceProcAddr>,
    automatic_destroy: bool,
) -> Option<pfn::GetInstanceProcAddr> {
    return Loader::hook_get_instance_proc_addr(func_in, automatic_destroy);
}

#[no_mangle]
pub extern "C" fn netvr_manual_destroy_instance(instance_handle: XrInstance) {
    Loader::netvr_manual_destroy_instance(instance_handle);
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(func: log::LoggerFn) {
    log::set_logger(func)
}
