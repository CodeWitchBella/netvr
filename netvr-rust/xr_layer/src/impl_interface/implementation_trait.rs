use super::*;

pub trait LayerImplementation {
    fn new(instance: &openxr::Instance) -> Self;

    fn sync_actions(&self, sync_info: &SyncActions) -> Result<(), openxr_sys::Result> {
        sync_info.sync()
    }
}
