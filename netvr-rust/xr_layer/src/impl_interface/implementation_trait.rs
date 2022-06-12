use super::*;

pub trait LayerImplementation {
    fn new(instance: &openxr::Instance) -> Self;

    fn sync_actions(&self, input: SyncActions) -> Result<(), openxr_sys::Result> {
        input.sync()
    }

    fn create_action(&self, input: CreateAction) -> Result<(), openxr_sys::Result> {
        input.create_action()
    }
}
