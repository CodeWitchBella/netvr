use crate::{utils::ResultConvertible, UnsafeFrom, XrDebug, XrResult, XrStructChain};

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

    pub fn sync_info(&self) -> XrStructChain {
        unsafe { XrStructChain::from_ptr(self.sync_info) }
    }
}

impl std::fmt::Debug for SyncActions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut f = f.debug_struct("SyncActions");
        f.field("instance", &self.instance.as_raw());
        f.field("session", &self.session_handle.as_debug(&self.instance));
        f.field("sync_info", &self.sync_info().as_debug(&self.instance));
        f.finish()
    }
}
