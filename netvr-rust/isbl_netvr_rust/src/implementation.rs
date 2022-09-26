use xr_layer::{log::LogInfo, openxr, CreateAction, LayerImplementation, SyncActions, XrResult};

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
        LogInfo::string(format!("xrSyncActions {:#?} -> {:?}", input, result));
        // Here we should run xrGetActionState, update local copy of locally-tracked
        // devices and trigger send to the server.
        // Also, we should read latest data which arrived from the server and
        // copy it to their local copies which we provide to the application.
        result
    }

    fn create_action(&self, input: CreateAction) -> XrResult<()> {
        let result = input.create_action();
        LogInfo::string(format!("xrCreateAction {:?} -> {:?}", input, result));
        result
    }

    fn get_action_state_boolean(&self, input: xr_layer::GetActionStateBoolean) -> XrResult<()> {
        let result = input.get();
        LogInfo::string(format!(
            "xrGetActionStateBoolean {:?} -> {:?}",
            input, result
        ));
        result
    }

    fn get_action_state_float(&self, input: xr_layer::GetActionStateFloat) -> XrResult<()> {
        let result = input.get();
        LogInfo::string(format!("xrGetActionStateFloat {:?} -> {:?}", input, result));
        result
    }
}
