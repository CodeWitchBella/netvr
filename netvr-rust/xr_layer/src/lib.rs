#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
mod impl_interface;
pub mod loader;
mod loader_globals;
pub mod log;
mod xr_structures;
pub use impl_interface::*;
pub use openxr;
