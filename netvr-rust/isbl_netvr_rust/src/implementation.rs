use xr_layer::{log::LogInfo, openxr, LayerImplementation, SyncActions};

pub struct ImplementationInstance {}
impl LayerImplementation for ImplementationInstance {
    fn new(_lower: &openxr::Instance) -> Self {
        Self {}
    }

    fn sync_actions(&self, input: SyncActions) -> Result<(), openxr::sys::Result> {
        let result = input.sync();
        LogInfo::string(format!("xrSyncActions {:#?} -> {:?}", input, result));
        result
    }

    fn create_action(&self, input: xr_layer::CreateAction) -> Result<(), openxr::sys::Result> {
        let result = input.create_action();
        LogInfo::string(format!("xrSyncActions {:#?} -> {:?}", input, result));
        result
    }
}
