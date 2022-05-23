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
    // this is useful in unity editor case where we want to unload the library
    // on xrInstanceDestroy, but that is before the actual destroy is called.
    // This means that we:
    //  - call netvr_manual_destroy_instance
    //  - unload the dll
    //  - your game engine calls xrInstanceDestroy from runtime
    // If you aren't doing any dll re-loading you should set this to true, which
    // results in following steps:
    //  - your game engine calls xrInstanceDestroy from netvr
    //  - netvr performs appropriate cleanup
    //  - netvr calls runtime's xrInstanceDestroy
    // This all happens automatically.
    automatic_destroy: bool,
) -> Option<openxr_sys::pfn::GetInstanceProcAddr> {
    log::log("isbl_netvr_hook_get_instance_proc_addr");
    let func = match func_in {
        Some(f) => f,
        None => {
            log::log("xrGetInstanceProcAddr is null. Expected valid function pointer.");
            return None;
        }
    };

    let mut value = match super::xr_functions::load(func) {
        Ok(v) => v,
        Err(error) => {
            log::log_string(format!(
                "Failed to initialize. Disabling netvr layer.\n  Original error: {}",
                error
            ));
            return func_in;
        }
    };
    value.automatic_destroy = automatic_destroy;
    let mut w = FUNCTIONS.write().unwrap();
    *w = Some(value);
    return Some(my_get_instance_proc_addr);
}

#[no_mangle]
pub extern "C" fn netvr_manual_destroy_instance(instance_handle: openxr_sys::Instance) {
    let status = my_destroy_instance(instance_handle);
    if status == openxr_sys::Result::SUCCESS {
        log::log_string(format!(
            "Instance {} was successfully destroyed",
            instance_handle.into_raw()
        ));
    } else {
        log::log_string(format!(
            "Instance {} was destroyed with non-zero status {}",
            instance_handle.into_raw(),
            crate::xr_functions::decode_xr_result(status)
        ));
    }
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
        ($t: ty, $func: expr) => {{
            if stringify!($t)[5..] == name[2..] {
                log::log_string(format!(
                    "xrGetInstanceProcAddr: Returning {} for {}",
                    stringify!($func),
                    name
                ));
                unsafe { *function = Some(std::mem::transmute($func as $t)) };
                return openxr_sys::Result::SUCCESS;
            }
        }};
    }

    if instance_handle == openxr_sys::Instance::NULL {
        check!(pfn::CreateInstance, my_create_instance);
        // "xrEnumerateInstanceExtensionProperties"
        // "xrEnumerateApiLayerProperties"
        return fallback!();
    }

    let r = INSTANCES.read().unwrap();
    let handle = instance_handle.into_raw();
    if (*r).get(&handle).is_none() {
        // do not return overrides for uninitialized instances
        return fallback!();
    };

    if name == "xrDestroyInstance" {
        let fns = get_functions!("xrGetInstanceProcAddr");
        // do not hook xrDestroyInstance if automatic_destroy is disabled
        if fns.automatic_destroy {
            check!(pfn::DestroyInstance, my_destroy_instance);
        } else {
            log::log("Skipping automatic destroy registration. You'll have to call netvr_manual_destroy_instance manually.");
        }
    }

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

extern "system" fn my_destroy_instance(
    instance_handle: openxr_sys::Instance,
) -> openxr_sys::Result {
    let mut w = INSTANCES.write().unwrap();
    let handle = instance_handle.into_raw();
    let instance = match (*w).get(&handle) {
        Some(v) => *v,
        None => {
            log::log_string(format!(
                "Can't find instance with handle {}. Maybe it was destroyed already?",
                handle
            ));
            return openxr_sys::Result::ERROR_HANDLE_INVALID;
        }
    };
    let result = unsafe { (instance.destroy_instance)(instance_handle) };
    (*w).remove(&handle);
    return result;
}
