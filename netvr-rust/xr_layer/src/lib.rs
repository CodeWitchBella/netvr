#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
mod impl_interface;
mod loader;
pub mod log;
mod sized_value_iterator;
mod xr_debug;
mod xr_listings;
mod xr_struct;
mod xr_struct_chain;
pub use impl_interface::*;
pub use loader::*;
// note that I am not reexporting openxr, since it is dangerous because most of
// its features implement Drop which we do not want as a openxr layer.
pub use openxr as safe_openxr;
pub use openxr::{raw, Entry, RawEntry};
pub use openxr_sys as sys;
pub use openxr_sys::pfn;
pub use sized_value_iterator::*;
pub use utils::*;
pub use xr_debug::*;
pub use xr_listings::*;
pub use xr_struct::*;
pub use xr_struct_chain::*;
