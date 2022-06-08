use xr_layer::loader::{ImplementationTrait, LowerLayer};

pub struct ImplementationInstance {}
impl ImplementationTrait for ImplementationInstance {
    fn new(_lower: &LowerLayer) -> Self {
        Self {}
    }
}
