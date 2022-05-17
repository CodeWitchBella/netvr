use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn netvr_set_logger(
    func: unsafe extern "C" fn(*const c_char)
) {
    println!("Hello world from Rust!");
    unsafe {
        func("hello 2".as_ptr() as *const c_char);
    }
}
