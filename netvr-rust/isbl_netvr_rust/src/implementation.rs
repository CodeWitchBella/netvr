use xr_layer::loader::{ImplementationTrait, LowerLayer};

pub struct ImplementationInstance {}
impl ImplementationTrait for ImplementationInstance {
    fn new(lower: &LowerLayer) -> Self {
        Self {}
    }
}
