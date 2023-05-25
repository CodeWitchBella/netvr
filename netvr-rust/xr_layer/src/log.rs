use std::{
    backtrace::Backtrace,
    error::Error,
    ffi::{CStr, CString},
    sync::RwLock,
};

use super::utils;

pub type LoggerFn = Option<unsafe extern "C" fn(i32, utils::Cstr, utils::Cstr)>;
lazy_static! {
    static ref LOGGER: RwLock<LoggerFn> = RwLock::new(Option::None);
}

macro_rules! cstr {
    ($s:literal) => {{
        unsafe { std::mem::transmute::<_, &std::ffi::CStr>(concat!($s, "\0")) }
    }};
}
const EMPTY_CSTR: &std::ffi::CStr = cstr!("");

#[derive(Clone, Copy)]
enum Level {
    Trace = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Panic = 5,
}

fn _cstr(level: Level, text: utils::Cstr) {
    let r = LOGGER.read().unwrap();
    match *r {
        Some(f) => {
            if level as i32 >= Level::Error as i32 {
                let backtrace = Backtrace::capture();
                let backtrace_cstring = CString::new(format!("{backtrace:?}")).unwrap();
                unsafe { f(level as i32, text, backtrace_cstring.as_ptr()) }
            } else {
                unsafe { f(level as i32, text, EMPTY_CSTR.as_ptr()) }
            }
        }
        None => {
            let c_str = unsafe { CStr::from_ptr(text) };
            match c_str.to_str() {
                Ok(v) => {
                    println!("[{}] {}", level as i32, v);
                }
                Err(error) => {
                    println!("Fallback printing failed: {}", error.source().unwrap());
                }
            };
        }
    }
}

fn _str(level: Level, text: &str) {
    let cstr = CString::new(text).unwrap();
    _cstr(level, cstr.as_ptr());
}

fn _string(level: Level, text: String) {
    let cstr = CString::new(text).unwrap();
    _cstr(level, cstr.as_ptr());
}

/// Creates a exported object for logging at a specific level.
macro_rules! implement {
    ($id:ident, $level:expr) => {
        pub struct $id {}
        #[allow(dead_code)]
        impl $id {
            pub fn cstr(text: utils::Cstr) {
                _cstr($level, text);
            }

            pub fn str(text: &str) {
                _str($level, text);
            }

            pub fn string(text: String) {
                _string($level, text);
            }
        }
    };
}
implement!(LogTrace, Level::Trace);
implement!(LogInfo, Level::Info);
implement!(LogWarn, Level::Warn);
implement!(LogError, Level::Error);
implement!(LogPanic, Level::Panic);

/// Change the global logger used by the above functions.
pub fn set_logger(func: LoggerFn) {
    println!("Hello world from Rust!");
    {
        let mut w = LOGGER.write().unwrap();
        *w = func;
    }
    LogInfo::str("Logger was setup and seems to be working correctly.");
}
