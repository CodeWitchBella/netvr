#![feature(array_chunks)]
#![feature(const_option)]
#[macro_use]
extern crate lazy_static;

use implementation::{read_remote_devices, tick};
use netvr_data::ReadRemoteDevicesInput;
use overrides::with_layer;
use std::panic;
use std::{alloc, backtrace::Backtrace};
use tracing::{span, Level};
use xr_layer::log::LogInfo;
use xr_wrap::{ResultConvertible, XrWrapError};

//use implementation::ImplementationInstance;
use xr_layer::{
    log::{self, LogPanic},
    pfn, sys,
};

mod implementation;
mod instance;
mod overrides;
mod xr_wrap;

/// this gets called from unity to give us option to override basically any openxr function
/// it only calls into loader and is what injects our safe rust implementation
#[no_mangle]
pub extern "C" fn netvr_hook_get_instance_proc_addr(
    func_in: Option<pfn::GetInstanceProcAddr>,
    manual_unhook: bool,
) -> Option<pfn::GetInstanceProcAddr> {
    func_in.map(|func| overrides::init(func, manual_unhook))
}

/// Deinitializes netvr. Needs to be called only if manual_unhook was specified
/// for hook function. Must be called after everything openxr-related is already
/// finished, but possibly before xrDestroyInstance.
#[no_mangle]
pub extern "C" fn netvr_unhook() {
    overrides::deinit()
}

/// Sets logger function to be used by the layer. This allows you to surface
/// netvr logs to for example unity editor.
#[no_mangle]
pub extern "C" fn netvr_set_logger(func: log::LoggerFn) {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::capture();
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

/// Should be called periodically. Used to do network upkeep separate from openxr's
/// rendering loop.
#[no_mangle]
pub extern "C" fn netvr_tick(instance_handle: sys::Instance) {
    xr_wrap::xr_wrap(|| {
        with_layer(instance_handle, |instance| {
            let _span = span!(Level::TRACE, "netvr_tick").entered();
            tick(instance)
        })
    });
}

fn bincode_abi<'de, O, Input, Output>(
    length: *mut u32,
    data: *mut *mut u8,
    function: O,
) -> sys::Result
where
    O: FnOnce(Input) -> Result<Output, XrWrapError>,
    O: std::panic::UnwindSafe,
    Input: serde::Deserialize<'de>,
    Output: serde::Serialize,
{
    xr_wrap::xr_wrap(|| {
        let slice = unsafe { std::slice::from_raw_parts(*data, { *length }.try_into()?) };
        let input: Input = bincode::deserialize(slice)?;

        let output = function(input)?;
        let encoded: Vec<u8> = bincode::serialize(&output)?;
        let u32len: u32 = encoded.len().try_into()?;

        let layout = alloc::Layout::from_size_align(encoded.len(), 1)?;
        unsafe {
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                return sys::Result::ERROR_OUT_OF_MEMORY.into_result();
            }
            std::ptr::copy(encoded.as_ptr(), ptr, encoded.len());
            length.write(u32len);
            data.write(ptr);
        };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn netvr_cleanup(length: u32, data: *mut u8) {
    let layout = alloc::Layout::from_size_align(length.try_into().unwrap(), 1).unwrap();
    unsafe {
        alloc::dealloc(data.try_into().unwrap(), layout);
    }
}

unsafe extern "C" fn netvr_read_remote_devices(
    length: *mut u32,
    data: *mut *mut u8,
) -> sys::Result {
    bincode_abi(length, data, |input: ReadRemoteDevicesInput| {
        with_layer(input.instance, read_remote_devices)
    })
}

#[no_mangle]
pub unsafe extern "C" fn netvr_get_fn(
    name_cstr: *const std::ffi::c_char,
    function: *mut Option<unsafe extern "C" fn(*mut u32, *mut *mut u8) -> sys::Result>,
) {
    LogInfo::cstr(name_cstr);
    function.write(None);
    if let Ok(name) = unsafe { std::ffi::CStr::from_ptr(name_cstr) }.to_str() {
        LogInfo::string(format!("Getting {name:?}"));
        match name {
            "read_remote_devices" => {
                function.write(Some(netvr_read_remote_devices));
            }
            default => {}
        };
    }
}
