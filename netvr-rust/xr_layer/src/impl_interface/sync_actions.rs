use crate::{utils::ResultConvertible, XrDebug, XrResult};

pub struct SyncActions {
    pub(crate) instance: openxr::Instance,
    pub(crate) session_handle: openxr_sys::Session,
    pub(crate) sync_info: *const openxr_sys::ActionsSyncInfo,
}

impl SyncActions {
    pub fn sync(&self) -> XrResult<()> {
        unsafe { (self.instance.fp().sync_actions)(self.session_handle, self.sync_info) }
            .into_result()
    }

    pub fn instance(&self) -> &openxr::Instance {
        &self.instance
    }
}

impl std::fmt::Debug for SyncActions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SyncActions")
            .field("instance", &self.instance.as_raw())
            .field("session", &self.session_handle.xr_debug(&self.instance))
            .field(
                "sync_info",
                &unsafe { self.sync_info.as_ref() }.xr_debug(&self.instance),
            )
            .finish()
    }
}
