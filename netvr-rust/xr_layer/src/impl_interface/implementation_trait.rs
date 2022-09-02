use super::*;

pub trait LayerImplementation {
    fn new(instance: &openxr::Instance) -> Self;

    fn sync_actions(&self, input: SyncActions) -> XrResult<()> {
        input.sync()
    }

    fn create_action(&self, input: CreateAction) -> XrResult<()> {
        input.create_action()
    }
}
