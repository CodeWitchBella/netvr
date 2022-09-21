#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
mod impl_interface;
pub mod loader;
mod loader_globals;
pub mod log;
mod xr_debug;
mod xr_iterator;
mod xr_struct;
pub use impl_interface::*;
pub use openxr;
pub use xr_debug::*;
pub use xr_iterator::*;
pub use xr_struct::*;
