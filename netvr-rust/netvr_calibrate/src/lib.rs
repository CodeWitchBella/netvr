mod calibrate;
mod input;

#[cfg(not(target = "wasm32"))]
pub use calibrate::*;
#[cfg(not(target = "wasm32"))]
pub use input::*;

#[cfg(crate_type = "cdylib")]
#[no_mangle]
pub extern "C" fn calibrate() {
    println!("Hello, world!");
}
