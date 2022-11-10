use std::sync::{Arc, Mutex};
use std::{error::Error, panic, sync::PoisonError};
use tracing::{dispatcher, Dispatch, Level};
use tracing_chrome::{ChromeLayerBuilder, EventOrSpan, FlushGuard, TraceStyle};
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
#[derive(Debug)]
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

#[derive(Clone)]
pub(crate) struct Trace {
    pub(self) dispatch: tracing::Dispatch,
    pub(self) _flush_guard: Arc<Mutex<FlushGuard>>,
}

impl Trace {
    pub(crate) fn new() -> Self {
        let (chrome_layer, trace_flush_guard) = ChromeLayerBuilder::new()
            .include_args(true)
            //.trace_style(TraceStyle::Async)
            .include_locations(true)
            /*  .name_fn(Box::new(|event_or_span| match event_or_span {
                EventOrSpan::Event(ev) => ev
                    .metadata()
                    .fields()
                    .field("message")
                    .map_or_else(|| ev.metadata().name().into(), |val| val.to_string()),
                EventOrSpan::Span(s) => s.metadata().name().into(),
            }))*/
            .build();

        Self {
            dispatch: Dispatch::new(
                FmtSubscriber::builder()
                    // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
                    // will be written to stdout.
                    .with_max_level(Level::TRACE)
                    // completes the builder.
                    .finish()
                    .with(chrome_layer),
            ),
            _flush_guard: Arc::new(Mutex::new(trace_flush_guard)),
        }
    }

    pub(crate) fn wrap<T>(&self, f: impl FnOnce() -> T) -> T {
        dispatcher::with_default(&self.dispatch, f)
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
