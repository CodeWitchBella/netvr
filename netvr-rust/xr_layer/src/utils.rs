use std::{os::raw::c_char, panic};

use crate::{
    decode_xr_result,
    log::{LogPanic, LogWarn},
};
pub type Cstr = *const c_char;

#[macro_export]
macro_rules! internal_screaming {
    () => {
        panic!("AAAAAAAAAAAaaaaaaaaaaaaaaaaaaaaaaaaa");
    };
}

fn print_panic(panic: Box<dyn std::any::Any + Send>) {
    match panic.downcast::<String>() {
        Ok(cause) => {
            LogPanic::string(format!(
                "Caught panic. This is probably a bug. cause: {}",
                cause
            ));
        }
        Err(err) => {
            LogPanic::string(format!(
                "Caught panic. This is probably a bug. cause: {:?}",
                err
            ));
        }
    };
}

pub fn xr_wrap<O>(function: O) -> openxr_sys::Result
where
    O: FnOnce() -> Result<(), openxr_sys::Result>,
    O: std::panic::UnwindSafe,
{
    let maybe_panicked = panic::catch_unwind(function);
    match maybe_panicked {
        Ok(result) => match result {
            Ok(()) => openxr_sys::Result::SUCCESS,
            Err(v) => v,
        },
        Err(panic) => {
            print_panic(panic);
            openxr_sys::Result::ERROR_RUNTIME_FAILURE
        }
    }
}

pub(crate) trait ResultConvertible {
    fn into_result(self) -> Result<(), openxr_sys::Result>;
}

impl ResultConvertible for openxr_sys::Result {
    fn into_result(self) -> Result<(), openxr_sys::Result> {
        if self == openxr_sys::Result::SUCCESS {
            Ok(())
        } else {
            Err(self)
        }
    }
}

pub(crate) trait ResultToWarning {
    fn warn_on_err(self, function_name: &'static str);
}

impl ResultToWarning for Result<(), openxr_sys::Result> {
    fn warn_on_err(self, function_name: &'static str) {
        if let Err(error) = self {
            LogWarn::string(format!(
                "Function {} failed with result {}",
                function_name,
                decode_xr_result(error)
            ))
        }
    }
}
