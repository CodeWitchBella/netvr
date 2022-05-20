use crate::log;
use crate::xr_functions::{self, XrFunctions, XrInstanceFunctions};

use openxr_sys::pfn;
use std::collections::hash_map::HashMap;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::RwLock;

lazy_static! {
    // store get_instance_proc_addr
    static ref FUNCTIONS: RwLock<Option<XrFunctions>> = RwLock::new(Option::None);
    static ref INSTANCES: RwLock<HashMap<u64, XrInstanceFunctions>> = RwLock::new(HashMap::new());
}

macro_rules! get_functions {
    ($caller: expr) => {{
        let caller: &str = $caller;
        let r = FUNCTIONS.read().unwrap();
        match *r {
            Some(v) => v,
            None => {
                log::log_string(format!(
                    "{} was called before setting up pointer to xrGetInstanceProcAddr",
                    caller
                ));
                return openxr_sys::Result::ERROR_RUNTIME_FAILURE;
            }
        }
    }};
}

// this gets called from unity to give us option to override basically any openxr function
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<openxr_sys::pfn::GetInstanceProcAddr>,
) -> Option<openxr_sys::pfn::GetInstanceProcAddr> {
    log::log("isbl_netvr_hook_get_instance_proc_addr");
    let func = match func_in {
        Some(f) => f,
        None => {
            log::log("xrGetInstanceProcAddr is null. Expected valid function pointer.");
            return None;
        }
    };

    let value = match super::xr_functions::load(func) {
        Ok(v) => v,
        Err(error) => {
            log::log_string(format!(
                "Failed to initialize. Disabling netvr layer.\n  Original error: {}",
                error
            ));
            return func_in;
        }
    };
    let mut w = FUNCTIONS.write().unwrap();
    *w = Some(value);
    return Some(my_get_instance_proc_addr);
}

// here we can return something different to override any openxr function
extern "system" fn my_get_instance_proc_addr(
    instance_handle: openxr_sys::Instance,
    name_ptr: *const c_char,
    function: *mut Option<openxr_sys::pfn::VoidFunction>,
) -> openxr_sys::Result {
    macro_rules! fallback {
        () => {{
            let fns = get_functions!("xrGetInstanceProcAddr");
            unsafe { (fns.get_instance_proc_addr)(instance_handle, name_ptr, function) }
        }};
    }

    log::log_cstr(name_ptr);
    let name = match unsafe { CStr::from_ptr(name_ptr) }.to_str() {
        Ok(val) => val,
        Err(error) => {
            log::log_string(format!(
                "xrGetInstanceProcAddr failed to parse name as UTF8. Netvr won't intercept this call. Error: {}",
                error.source().unwrap(),
            ));
            return fallback!();
        }
    };

    macro_rules! check {
        ($t: ty, $name: expr, $func: expr) => {{
            if name == $name {
                unsafe { *function = Some(std::mem::transmute($func as $t)) };
                return openxr_sys::Result::SUCCESS;
            }
        }};
    }

    if instance_handle == openxr_sys::Instance::NULL {
        check!(pfn::CreateInstance, "xrCreateInstance", my_create_instance);
        // "xrEnumerateInstanceExtensionProperties"
        // "xrEnumerateApiLayerProperties"
        return fallback!();
    }

    let r = INSTANCES.read().unwrap();
    let handle = instance_handle.into_raw();
    let instance = match (*r).get(&handle) {
        Some(v) => *v,
        None => {
            return fallback!();
        }
    };

    // TODO: handle instance functions
    // instance.destroy_instance

    return fallback!();
}

extern "system" fn my_create_instance(
    create_info: *const openxr_sys::InstanceCreateInfo,
    instance_ptr: *mut openxr_sys::Instance,
) -> openxr_sys::Result {
    let fns = get_functions!("xrCreateInstance");

    let result = unsafe { (fns.create_instance)(create_info, instance_ptr) };
    let instance: openxr_sys::Instance = unsafe { *instance_ptr };
    if result != openxr_sys::Result::SUCCESS {
        log::log_string(format!(
            "Underlying xrCreateInstance returned non-success error code {}. Netvr won't be enabled for instance {}.",
            xr_functions::decode_xr_result(result),
            instance.into_raw(),
        ));
        return result;
    };
    let value = match super::xr_functions::load_instance(instance, fns.get_instance_proc_addr) {
        Ok(v) => v,
        Err(error) => {
            log::log_string(format!(
                "load_instance failed with error {}. Netvr won't be enabled for instance {}.",
                error,
                instance.into_raw(),
            ));
            return result;
        }
    };

    let mut w = INSTANCES.write().unwrap();
    (*w).insert(instance.into_raw(), value);
    return result;
}
