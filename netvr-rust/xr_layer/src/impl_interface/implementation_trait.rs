use super::*;

pub trait LayerImplementation: std::marker::Send {
    fn should_override(action: &str) -> bool;

    fn sync_actions(_: SyncActions) -> XrResult<()> {
        panic!("Method is set to be overriden but is not implemented")
    }

    fn create_action(_: CreateAction) -> XrResult<()> {
        panic!("Method is set to be overriden but is not implemented")
    }

    fn get_action_state_boolean(_: GetActionStateBoolean) -> XrResult<()> {
        panic!("Method is set to be overriden but is not implemented")
    }

    fn get_action_state_float(_: GetActionStateFloat) -> XrResult<()> {
        panic!("Method is set to be overriden but is not implemented")
    }

    fn get_action_state_vector2f(_: GetActionStateVector2f) -> XrResult<()> {
        panic!("Method is set to be overriden but is not implemented")
    }

    fn get_action_state_pose(_: GetActionStatePose) -> XrResult<()> {
        panic!("Method is set to be overriden but is not implemented")
    }
}
