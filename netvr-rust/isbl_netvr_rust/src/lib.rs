#[macro_use]
extern crate lazy_static;

use binary_layout::prelude::*;
use implementation::tick;
use overrides::with_layer;
use std::backtrace::Backtrace;
use std::panic;
use tracing::{span, Level};
use xr_layer::log::LogError;
use xr_wrap::ResultConvertible;

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

define_layout!(device, LittleEndian, {
    id: u32,
    x: f32,
    y: f32,
    z: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
});

#[no_mangle]
pub unsafe extern "C" fn netvr_read_remote_device_data(
    _instance_handle: sys::Instance,
    length: u32,
    data: *mut u8,
) -> sys::Result {
    xr_wrap::xr_wrap(|| {
        let device_size: usize = device::SIZE.unwrap();
        let length: usize = length.try_into()?;
        if length % device_size != 0 {
            LogError::str("Not divisible by per_item");
            return sys::Result::ERROR_SIZE_INSUFFICIENT.into_result();
        }

        let slice = &mut *std::ptr::slice_from_raw_parts_mut(data, length);
        for i in 0..(length / device_size) {
            let mut view = device::View::new(&mut slice[i * device_size..device_size]);
            view.id_mut().write(1);
            view.x_mut().write(0.0);
            view.y_mut().write(1.0);
            view.z_mut().write(0.0);
            view.qx_mut().write(0.0);
            view.qy_mut().write(0.0);
            view.qz_mut().write(0.0);
            view.qw_mut().write(1.0);
        }
        sys::Result::SUCCESS.into_result()
    })
}

#[no_mangle]
pub extern "C" fn netvr_read_remote_device_count(_instance_handle: sys::Instance) -> u32 {
    1
}
