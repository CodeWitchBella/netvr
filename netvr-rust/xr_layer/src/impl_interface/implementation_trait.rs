use super::*;

pub trait LayerImplementation {
    fn new(instance: &openxr::Instance) -> Self;

    fn sync_actions(&self, input: SyncActions) -> XrResult<()> {
        input.sync()
    }

    fn create_action(&self, input: CreateAction) -> XrResult<()> {
        input.create_action()
    }

    fn get_action_state_boolean(&self, input: GetActionStateBoolean) -> XrResult<()> {
        input.get()
    }

    fn get_action_state_float(&self, input: GetActionStateFloat) -> XrResult<()> {
        input.get()
    }

    fn get_action_state_vector2f(&self, input: GetActionStateVector2f) -> XrResult<()> {
        input.get()
    }

    fn get_action_state_pose(&self, input: GetActionStatePose) -> XrResult<()> {
        input.get()
    }
}
