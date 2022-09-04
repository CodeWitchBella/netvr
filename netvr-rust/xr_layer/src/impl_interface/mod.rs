mod create_action;
mod debug_helpers;
mod implementation_trait;
mod sync_actions;

pub use create_action::*;
pub use implementation_trait::*;
pub use sync_actions::*;

pub type XrResult<T> = Result<T, openxr::sys::Result>;
