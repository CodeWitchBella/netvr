use xr_layer::{
    log::LogInfo, openxr, CreateAction, LayerImplementation, SyncActions, XrDebug, XrResult,
};

pub struct ImplementationInstance {
    lower: openxr::Instance,
}
impl LayerImplementation for ImplementationInstance {
    fn new(lower: &openxr::Instance) -> Self {
        Self {
            lower: lower.clone(),
        }
    }

    fn sync_actions(&self, input: SyncActions) -> XrResult<()> {
        let result = input.sync();
        LogInfo::string(format!("xrSyncActions {:?} -> {:?}", input, result));
        result
    }

    fn create_action(&self, input: CreateAction) -> XrResult<()> {
        let result = input.create_action();
        LogInfo::string(format!("xrCreateAction {:?} -> {:?}", input, result));
        result
    }
}
