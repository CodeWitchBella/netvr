use super::log;
use super::xr_functions::XrFunctionsNoInstance;
use std::os::raw::c_char;
use std::sync::RwLock;

lazy_static! {
    // store get_instance_proc_addr
    static ref FUNCTIONS: RwLock<Option<XrFunctionsNoInstance>> = RwLock::new(Option::None);
}

// this gets called from unity to give us option to override basically any openxr function
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func: Option<openxr_sys::pfn::GetInstanceProcAddr>,
) -> Option<openxr_sys::pfn::GetInstanceProcAddr> {
    log::log("isbl_netvr_hook_get_instance_proc_addr");

    if func.is_none() {
        log::log("Its none");
    }

    let v = match func {
        Some(f) => super::xr_functions::load(f),
        None => {
            internal_screaming!();
        }
    };
    let mut w = FUNCTIONS.write().unwrap();
    *w = Some(v);
    return Some(my_get_instance_proc_addr);
}

// here we can return something different to override any openxr function
extern "system" fn my_get_instance_proc_addr(
    instance: openxr_sys::Instance,
    name: *const c_char,
    function: *mut Option<openxr_sys::pfn::VoidFunction>,
) -> openxr_sys::Result {
    log::log_cstr(name);
    let r = FUNCTIONS.read().unwrap();
    match &*r {
        Some(f) => unsafe { (f.get_instance_proc_addr)(instance, name, function) },
        None => openxr_sys::Result::ERROR_RUNTIME_FAILURE,
    }
}
