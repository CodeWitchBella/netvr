use std::{error::Error, panic, sync::PoisonError};

use xr_layer::{
    log::{LogError, LogPanic, LogTrace},
    sys,
};

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

/// Any override function can return this. It is useful to be able to use ?
/// operator on basically anything. Each error is associated with behavior
/// (like logging) and xr error code.
pub(crate) enum XrWrapError {
    /// Error that is expected to happen, or is outside layer's control.
    Expected(sys::Result),
    /// Internal error in the application. Corresponds to sys::Result::ERROR_RUNTIME_FAILURE
    Generic(Box<dyn Error>),
}

impl From<sys::Result> for XrWrapError {
    fn from(result: sys::Result) -> Self {
        Self::Expected(result)
    }
}

impl<T: 'static> From<PoisonError<T>> for XrWrapError
where
    T: Sized,
{
    fn from(error: PoisonError<T>) -> Self {
        Self::Generic(Box::new(error))
    }
}

/// Makes sure that layer never crashes. Catches panics. Also allows for more
/// ergonomic handle writing using ? operator.
pub(crate) fn xr_wrap<O>(function: O) -> sys::Result
where
    O: FnOnce() -> Result<(), XrWrapError>,
    O: std::panic::UnwindSafe,
{
    let maybe_panicked = panic::catch_unwind(function);
    match maybe_panicked {
        Ok(result) => match result {
            Ok(()) => sys::Result::SUCCESS,
            Err(err) => match err {
                XrWrapError::Expected(result) => result,
                XrWrapError::Generic(poison) => {
                    LogError::string(format!("Call failed with error: {:?}", poison));
                    sys::Result::ERROR_RUNTIME_FAILURE
                }
            },
        },
        Err(panic) => {
            print_panic(panic);
            sys::Result::ERROR_RUNTIME_FAILURE
        }
    }
}

pub(crate) fn xr_wrap_trace<O>(fn_name: &'static str, function: O) -> sys::Result
where
    O: FnOnce() -> Result<(), XrWrapError>,
    O: std::panic::UnwindSafe,
{
    let result = xr_wrap(function);
    let maybe_panicked = panic::catch_unwind(|| {
        LogTrace::string(format!("{} -> {:?}", fn_name, result));
    });
    if maybe_panicked.is_err() {
        LogError::str("Trace function panicked");
    }
    result
}

/// Utility trait to be able to do `result.into_result()?` inside the function.
pub(crate) trait ResultConvertible {
    fn into_result(self) -> Result<(), XrWrapError>;
}

impl ResultConvertible for sys::Result {
    fn into_result(self) -> Result<(), XrWrapError> {
        if self == Self::SUCCESS {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}
