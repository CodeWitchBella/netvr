use xr_layer::{loader::ImplementationTrait, openxr};

pub struct ImplementationInstance {}
impl ImplementationTrait for ImplementationInstance {
    fn new(_lower: &openxr::Instance) -> Self {
        Self {}
    }
}
