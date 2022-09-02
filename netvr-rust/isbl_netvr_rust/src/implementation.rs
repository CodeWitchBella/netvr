use xr_layer::{log::LogInfo, openxr, CreateAction, LayerImplementation, SyncActions, XrResult};

pub struct ImplementationInstance {}
impl LayerImplementation for ImplementationInstance {
    fn new(_lower: &openxr::Instance) -> Self {
        Self {}
    }

    fn sync_actions(&self, input: SyncActions) -> XrResult<()> {
        let result = input.sync();
        LogInfo::string(format!("xrSyncActions {:#?} -> {:?}", input, result));
        result
    }

    fn create_action(&self, input: CreateAction) -> XrResult<()> {
        let result = input.create_action();
        LogInfo::string(format!("xrCreateAction {:#?} -> {:?}", input, result));
        result
    }
}
