use super::utils;
use std::ffi::CString;
use std::sync::RwLock;

// region: logger
type LoggerFn = Option<unsafe extern "C" fn(utils::Cstr)>;
lazy_static! {
    static ref LOGGER: RwLock<LoggerFn> = RwLock::new(Option::None);
}

pub fn log_cstr(s: utils::Cstr) {
    let r = LOGGER.read().unwrap();
    match *r {
        Some(f) => unsafe { f(s) },
        None => {}
    }
}

pub fn log(s: &str) {
    let cstr = CString::new(s).unwrap();
    log_cstr(cstr.as_ptr());
}

pub fn log_string(s: String) {
    let cstr = CString::new(s).unwrap();
    log_cstr(cstr.as_ptr());
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(func: LoggerFn) {
    println!("Hello world from Rust!");
    {
        let mut w = LOGGER.write().unwrap();
        *w = func;
    }
    log("Hello there\n");
}
