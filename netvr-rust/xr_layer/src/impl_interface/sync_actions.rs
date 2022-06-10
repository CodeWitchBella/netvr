use std::fmt::Debug;

use crate::utils::ResultConvertible;

pub struct SyncActions {
    pub(crate) instance: openxr::Instance,
    pub(crate) session_handle: openxr_sys::Session,
    pub(crate) sync_info: *const openxr_sys::ActionsSyncInfo,
}

impl Debug for SyncActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionSyncData")
            .field("instance", &self.instance.as_raw())
            .field("session_handle", &self.session_handle)
            .field("sync_info", &self.sync_info)
            .finish()
    }
}

impl SyncActions {
    pub fn sync(&self) -> Result<(), openxr_sys::Result> {
        unsafe { (self.instance.fp().sync_actions)(self.session_handle, self.sync_info) }
            .into_result()
    }

    pub fn instance(&self) -> &openxr::Instance {
        &self.instance
    }
}
