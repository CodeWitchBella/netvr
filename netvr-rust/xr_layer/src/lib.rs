#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
pub mod loader;
pub mod log;
mod xr_functions;
mod xr_structures;
pub use openxr_sys::pfn;
pub use openxr_sys::Instance as XrInstance;
