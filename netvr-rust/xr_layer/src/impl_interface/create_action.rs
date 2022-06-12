use std::fmt::Debug;

use crate::utils::ResultConvertible;

pub struct CreateAction {
    pub(crate) instance: openxr::Instance,
    pub(crate) action_set_handle: openxr_sys::ActionSet,
    pub(crate) info: *const openxr_sys::ActionCreateInfo,
    pub(crate) out: *mut openxr_sys::Action,
}

impl Debug for CreateAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateAction")
            .field("action_set_handle", &self.action_set_handle)
            .field("info", &self.info)
            .field("out", &self.out)
            .finish()
    }
}

impl CreateAction {
    pub fn create_action(&self) -> Result<(), openxr_sys::Result> {
        unsafe { (self.instance.fp().create_action)(self.action_set_handle, self.info, self.out) }
            .into_result()
    }

    pub fn instance(&self) -> &openxr::Instance {
        &self.instance
    }
}
