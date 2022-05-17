use std::os::raw::c_char;

// region: logger
static mut LOGGER: unsafe extern "C" fn(*const c_char) = log_fallback;

extern "C" fn log_fallback(_arg: *const c_char) {}

fn log(s: &str) {
    unsafe {
        LOGGER(s.as_ptr() as *const c_char);
    }
}

#[no_mangle]
pub extern "C" fn netvr_set_logger(
    func: unsafe extern "C" fn(*const c_char)
) {
    println!("Hello world from Rust!");
    unsafe { LOGGER = func; }
    log("Hello there");
}
// endregion: logger

