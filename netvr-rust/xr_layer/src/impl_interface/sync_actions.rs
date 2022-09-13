use std::fmt::Debug;

use super::debug_helpers::InstanceDebug;
use crate::{utils::ResultConvertible, XrResult};

#[derive(Debug)]
pub struct SyncActions {
    pub(crate) instance: InstanceDebug,
    pub(crate) session_handle: openxr_sys::Session,
    pub(crate) sync_info: *const openxr_sys::ActionsSyncInfo,
}

impl SyncActions {
    pub fn sync(&self) -> XrResult<()> {
        unsafe { (self.instance.fp().sync_actions)(self.session_handle, self.sync_info) }
            .into_result()
    }

    pub fn instance(&self) -> &openxr::Instance {
        &self.instance.0
    }
}
