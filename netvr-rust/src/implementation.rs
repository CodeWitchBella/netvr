use crate::loader::ImplementationTrait;

pub struct ImplementationInstance {}
impl ImplementationTrait for ImplementationInstance {
    fn new() -> Self {
        Self {}
    }
}
