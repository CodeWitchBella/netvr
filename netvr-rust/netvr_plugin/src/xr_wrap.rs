use core::ffi::FromBytesUntilNulError;
#[cfg(not(target_os = "android"))]
use std::sync::{Arc, Mutex};
use std::{alloc::LayoutError, panic, sync::PoisonError};

use netvr_data::bincode;
use thiserror::Error;
use tracing::{dispatcher, span::EnteredSpan, Dispatch, Level, Span};
#[cfg(not(target_os = "android"))]
use tracing_chrome::{ChromeLayerBuilder, FlushGuard};
#[cfg(not(target_os = "android"))]
use tracing_subscriber::prelude::*;
use tracing_subscriber::FmtSubscriber;
use xr_layer::{
    log::{LogError, LogPanic},
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
#[derive(Debug, Error)]
pub(crate) enum XrWrapError {
    /// Error that is expected to happen, or is outside layer's control.
    #[error("OpenXR error: {0:?}")]
    Expected(sys::Result),
    /// Internal error in the application. Corresponds to
    /// sys::Result::ERROR_RUNTIME_FAILURE
    #[error("Internal Error")]
    Generic(anyhow::Error),
}

impl From<sys::Result> for XrWrapError {
    fn from(result: sys::Result) -> Self {
        Self::Expected(result)
    }
}

impl<T> From<PoisonError<T>> for XrWrapError {
    fn from(error: PoisonError<T>) -> Self {
        Self::Generic(anyhow::Error::msg(error.to_string()))
    }
}

impl From<std::num::TryFromIntError> for XrWrapError {
    fn from(error: std::num::TryFromIntError) -> Self {
        Self::Generic(error.into())
    }
}

impl From<Box<bincode::ErrorKind>> for XrWrapError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Self::Generic(error.into())
    }
}

impl From<LayoutError> for XrWrapError {
    fn from(error: LayoutError) -> Self {
        Self::Generic(error.into())
    }
}

impl From<anyhow::Error> for XrWrapError {
    fn from(error: anyhow::Error) -> Self {
        Self::Generic(error)
    }
}

impl From<std::ffi::NulError> for XrWrapError {
    fn from(error: std::ffi::NulError) -> Self {
        Self::Generic(error.into())
    }
}

impl From<std::str::Utf8Error> for XrWrapError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Generic(error.into())
    }
}

impl From<FromBytesUntilNulError> for XrWrapError {
    fn from(error: FromBytesUntilNulError) -> Self {
        Self::Generic(error.into())
    }
}

impl From<xr_layer::StringParseError> for XrWrapError {
    fn from(error: xr_layer::StringParseError) -> Self {
        Self::Generic(error.into())
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
                XrWrapError::Generic(error) => {
                    LogError::string(format!("Call failed with error: {:?}", error));
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

#[derive(Clone)]
pub(crate) struct Trace {
    pub(self) dispatch: tracing::Dispatch,
    #[cfg(not(target_os = "android"))]
    pub(self) _flush_guard: Arc<Mutex<FlushGuard>>,
}

impl Trace {
    pub(crate) fn new() -> Self {
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn,
            // etc.) will be written to stdout.
            .with_max_level(Level::TRACE)
            // completes the builder.
            .finish();
        #[cfg(not(target_os = "android"))]
        {
            let (chrome_layer, trace_flush_guard) = ChromeLayerBuilder::new()
                .include_args(true)
                //.trace_style(tracing_chrome::TraceStyle::Async)
                .include_locations(true)
                // .name_fn(Box::new(|event_or_span| match event_or_span {
                //     EventOrSpan::Event(ev) => ev
                //         .metadata()
                //         .fields()
                //         .field("message")
                //         .map_or_else(|| ev.metadata().name().into(), |val| val.to_string()),
                //     EventOrSpan::Span(s) => s.metadata().name().into(),
                // }))
                .build();
            let dispatch = Dispatch::new(subscriber.with(chrome_layer));
            Self {
                dispatch,
                _flush_guard: Arc::new(Mutex::new(trace_flush_guard)),
            }
        }

        #[cfg(target_os = "android")]
        Self {
            dispatch: Dispatch::new(subscriber),
        }
    }

    pub(crate) fn wrap<T>(&self, f: impl FnOnce() -> T) -> T {
        dispatcher::with_default(&self.dispatch, f)
    }
}

pub(crate) trait RecordDebug {
    fn record_debug<T>(&self, field: &'static str, debug: T)
    where
        T: std::fmt::Debug;
}

impl RecordDebug for Span {
    fn record_debug<T>(&self, field: &'static str, debug: T)
    where
        T: std::fmt::Debug,
    {
        self.record(field, tracing::field::debug(debug));
    }
}

impl RecordDebug for EnteredSpan {
    fn record_debug<T>(&self, field: &'static str, debug: T)
    where
        T: std::fmt::Debug,
    {
        self.record(field, tracing::field::debug(debug));
    }
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
