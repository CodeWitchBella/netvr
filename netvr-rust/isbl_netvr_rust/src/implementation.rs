use xr_layer::{log::LogInfo, openxr, LayerImplementation, SyncActions};

pub struct ImplementationInstance {}
impl LayerImplementation for ImplementationInstance {
    fn new(_lower: &openxr::Instance) -> Self {
        Self {}
    }

    fn sync_actions(&self, sync_info: &SyncActions) -> Result<(), openxr::sys::Result> {
        let result = sync_info.sync();
        LogInfo::string(format!("xrSyncActions {:#?} -> {:?}", sync_info, result));
        result
    }
}
