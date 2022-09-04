use std::fmt::Debug;

use super::debug_helpers::InstanceDebug;
use crate::{utils::ResultConvertible, XrResult};

#[derive(Debug)]
pub struct CreateAction {
    pub(crate) instance: InstanceDebug,
    pub(crate) action_set_handle: openxr_sys::ActionSet,
    pub(crate) info: *const openxr_sys::ActionCreateInfo,
    pub(crate) out: *mut openxr_sys::Action,
}

impl CreateAction {
    pub fn create_action(&self) -> XrResult<()> {
        unsafe { (self.instance.fp().create_action)(self.action_set_handle, self.info, self.out) }
            .into_result()
    }

    pub fn instance(&self) -> &openxr::Instance {
        &self.instance.v
    }
}
