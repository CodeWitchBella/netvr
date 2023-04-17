use std::{collections::HashMap, error::Error, ffi::CStr, os::raw::c_char};

use openxr_sys::pfn;

use crate::{log::LogWarn, sys, utils::ResultConvertible, xr_listings::FnPtr};

pub struct Layer {
    map: HashMap<&'static str, pfn::VoidFunction>,
    func: pfn::GetInstanceProcAddr,
}

impl Layer {
    /// Creates new instance of Layer to be built
    ///
    /// # Safety
    ///
    /// `get_instance_proc_addr` must be valid. This is usually fine if you are
    /// passing in a pointer you received from somewhere else. But it is
    /// probably a good idea to check for null anyway? As a treat.
    pub unsafe fn new(func: pfn::GetInstanceProcAddr) -> Self {
        Self {
            map: HashMap::default(),
            func,
        }
    }
}

impl Layer {
    /// Set up function override to be called instead of runtime's function.
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
    ) -> Result<Option<pfn::VoidFunction>, sys::Result> {
        let fallback = || {
            let mut function: Option<openxr_sys::pfn::VoidFunction> = None;
            let result = unsafe {
                self.get_instance_proc_addr_runtime(instance_handle, name_ptr, &mut function)
            }
            .into_result();
            if let Err(err) = result {
                Err(err)
            } else {
                Ok(function)
            }
        };

        let name = match parse_input_string(name_ptr) {
            Some(val) => val,
            None => {
                return fallback();
            }
        };

        if !name.starts_with("xr") {
            return fallback();
        }

        match self.map.get(name) {
            Some(&fun) /* heh. */ => {
                Ok(Some(fun))
            }
            None => fallback(),
        }
    }
}
