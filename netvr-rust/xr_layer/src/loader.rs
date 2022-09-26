use crate::{
    log::LogWarn,
    utils::{xr_wrap, ResultConvertible},
    xr_listings::FnPtr,
};

use openxr_sys::pfn;
use std::{collections::HashMap, error::Error, ffi::CStr, os::raw::c_char};

pub struct Layer {
    map: HashMap<&'static str, pfn::VoidFunction>,
    func: pfn::GetInstanceProcAddr,
}

impl Layer {
    /// Creates new instance of Layer to be built
    ///
    /// # Safety
    /// `get_instance_proc_addr` must be valid
    pub unsafe fn new(func: pfn::GetInstanceProcAddr) -> Self {
        Self {
            map: HashMap::default(),
            func,
        }
    }
}

impl Layer {
    pub fn add_override(&mut self, pfn: FnPtr) -> &mut Self {
        self.map.insert(pfn.value(), pfn.void_fn());
        self
    }
}

fn parse_input_string<'a>(name_ptr: *const c_char) -> Option<&'a str> {
    match unsafe { CStr::from_ptr(name_ptr) }.to_str() {
        Ok(val) => Some(val),
        Err(error) => {
            LogWarn::string(format!(
                "Failed to parse string input as UTF8. Error: {:?}",
                error.source()
            ));
            None
        }
    }
}

impl Layer {
    /// Just a wrapper around get_instance_proc_addr, does nothing extra.
    /// Useful in case something fails and we want to recover reasonably
    ///
    /// # Safety
    ///
    /// All pointers must be valid. This is probably fine if you are just
    /// passing in pointers you received, since OpenXR spec mandates all
    /// arguments are valid. And we are only providing same guarantees as the
    /// original function anyway.
    pub unsafe fn get_instance_proc_addr_runtime(
        &self,
        instance_handle: openxr_sys::Instance,
        name_ptr: *const c_char,
        function: *mut Option<openxr_sys::pfn::VoidFunction>,
    ) -> openxr_sys::Result {
        (self.func)(instance_handle, name_ptr, function)
    }

    /// here we can return something different to override any openxr function
    /// https://registry.khronos.org/OpenXR/specs/1.0/html/xrspec.html#function-pointers
    pub fn get_instance_proc_addr(
        &self,
        instance_handle: openxr_sys::Instance,
        name_ptr: *const c_char,
        function: *mut Option<openxr_sys::pfn::VoidFunction>,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            /*
            let fallback = || {
                let r = LOADER_ROOT.read().unwrap();
                let root = r.deref().as_ref().unwrap();
                unsafe { (root.fp().get_instance_proc_addr)(instance_handle, name_ptr, function) }
                    .into_result()
            };

            let name = match parse_input_string(name_ptr) {
                Some(val) => val,
                None => {
                    return fallback();
                }
            };

            if !name.starts_with("xr") {
                LogWarn::string(format!(
                    "xrGetInstanceProcAddr can only handle functions starting with xr. Got: {}",
                    name,
                ));
                return fallback();
            }

            macro_rules! check {
                ($t: ty, $func: ident) => {{
                    if stringify!($t)[5..] == name[2..] {
                        LogTrace::string(format!(
                            "xrGetInstanceProcAddr: Returning {} for {}",
                            stringify!($func),
                            name
                        ));
                        let func: $t = Self::$func; // guard that $func is of correct type
                        unsafe { *function = Some(std::mem::transmute(func)) };
                        return Ok(());
                    }
                }};
            }

            if instance_handle == openxr_sys::Instance::NULL {
                if "xrCreateInstance" == name {
                    let func: pfn::CreateInstance = Self::override_create_instance;
                    unsafe { *function = Some(std::mem::transmute(func)) };
                    Ok(())
                } else {
                    // There are also
                    //  - xrEnumerateApiLayerProperties
                    //  - xrEnumerateInstanceExtensionProperties
                    // but neither of them is of interest to me.
                    fallback()
                }
            } else if let Some(instance) = H_INSTANCE.read(&instance_handle) {
                // noop, but helps rust-analyzer to make sense of this mess
                let instance: Arc<openxr::raw::Instance> = instance;
                let v = LayerImplementation::should_override(name);
            } else {
                fallback()
            }
            */
            openxr_sys::Result::SUCCESS.into_result()
        })
    }
}
