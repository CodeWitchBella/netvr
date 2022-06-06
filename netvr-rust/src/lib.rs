#[macro_use]
mod utils;
mod xr_functions;
mod xr_structures;

#[macro_use]
extern crate lazy_static;

mod log;
pub use log::netvr_set_logger;

mod loader;

mod implementation;
pub use implementation::{netvr_hook_get_instance_proc_addr, netvr_manual_destroy_instance};
