use super::log;
use std::ffi::CString;

use openxr_sys::pfn;

pub struct XrFunctionsNoInstance {
    pub get_instance_proc_addr: pfn::GetInstanceProcAddr,
    pub enumerate_instance_extension_properties: pfn::EnumerateInstanceExtensionProperties,
    pub enumerate_api_layer_properties: pfn::EnumerateApiLayerProperties,
    pub create_instance: pfn::CreateInstance,
}

macro_rules! find_and_cast {
    ($func: expr, $name: expr, $t:ty) => {{
        let raw = call_get_instance_proc_addr($func, $name);
        unsafe {
            match raw {
                Some(f) => std::mem::transmute::<pfn::VoidFunction, $t>(f),
                None => {
                    log::log_string(format!("Failed to load {}", $name));
                    internal_screaming!();
                }
            }
        }
    }};
}

pub fn load(func: openxr_sys::pfn::GetInstanceProcAddr) -> XrFunctionsNoInstance {
    let functions = XrFunctionsNoInstance {
        get_instance_proc_addr: func,
        enumerate_instance_extension_properties: find_and_cast!(
            func,
            "xrEnumerateInstanceExtensionProperties",
            pfn::EnumerateInstanceExtensionProperties
        ),
        enumerate_api_layer_properties: find_and_cast!(
            func,
            "xrEnumerateApiLayerProperties",
            pfn::EnumerateApiLayerProperties
        ),
        create_instance: find_and_cast!(func, "xrCreateInstance", pfn::CreateInstance),
    };
    return functions;
}

fn call_get_instance_proc_addr(
    func: openxr_sys::pfn::GetInstanceProcAddr,
    name: &str,
) -> Option<pfn::VoidFunction> {
    let mut function: Option<pfn::VoidFunction> = Option::None;
    let name_cstr = CString::new(name).unwrap();
    unsafe {
        let _status = func(
            openxr_sys::Instance::NULL,
            name_cstr.as_ptr(),
            &mut function,
        );
        // TODO: do something with status
    }
    return function;
}
