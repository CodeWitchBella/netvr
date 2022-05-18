use std::os::raw::c_char;

// region: logger
type LoggerFn = Option<unsafe extern "C" fn(*const c_char)>;
static mut LOGGER: LoggerFn = Option::None;

fn log_cstr(s: *const c_char) {
    let logger: LoggerFn;
    unsafe { logger = LOGGER; }
    match logger {
        Some(f) => unsafe { f(s); },
        None    => {},
    }
}

fn log(s: &str) {
    log_cstr(s.as_ptr() as *const c_char);
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(func: LoggerFn) {
    println!("Hello world from Rust!");
    unsafe { LOGGER = func; }
    log("Hello there\n");
}
// endregion: logger

// store get_instance_proc_addr
static mut ORIG_GET_INSTANCE_PROC_ADDR: Option<openxr_sys::pfn::GetInstanceProcAddr> = Option::None;

// this gets called from unity to give us option to override basically any openxr function
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(func: Option<openxr_sys::pfn::GetInstanceProcAddr>) -> openxr_sys::pfn::GetInstanceProcAddr
{
    log("isbl_netvr_hook_get_instance_proc_addr");
    unsafe { ORIG_GET_INSTANCE_PROC_ADDR = func; }
    return my_get_instance_proc_addr;
}

// here we can return something different to override any openxr function
extern "system" fn my_get_instance_proc_addr(
    instance: openxr_sys::Instance, 
    name: *const c_char, 
    function: *mut Option<openxr_sys::pfn::VoidFunction>
) -> openxr_sys::Result {
    log_cstr(name);
    let func: Option<openxr_sys::pfn::GetInstanceProcAddr>;
    unsafe { func = ORIG_GET_INSTANCE_PROC_ADDR; }
    match func {
        Some(f) => unsafe { return f(instance, name, function) },
        None    => openxr_sys::Result::ERROR_RUNTIME_FAILURE,
    }
}
