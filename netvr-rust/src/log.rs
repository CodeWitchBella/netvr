use std::os::raw::c_char;

// region: logger
type LoggerFn = Option<unsafe extern "C" fn(*const c_char)>;
static mut LOGGER: LoggerFn = Option::None;

pub fn log_cstr(s: *const c_char) {
    let logger: LoggerFn;
    unsafe {
        logger = LOGGER;
    }
    match logger {
        Some(f) => unsafe {
            f(s);
        },
        None => {}
    }
}

pub fn log(s: &str) {
    log_cstr(s.as_ptr() as *const c_char);
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(func: LoggerFn) {
    println!("Hello world from Rust!");
    unsafe {
        LOGGER = func;
    }
    log("Hello there\n");
}
