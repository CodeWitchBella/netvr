use xr_layer::raw;

pub(crate) struct Instance {
    pub(crate) pfn: raw::Instance,
}

impl Instance {
    pub(crate) fn new(pfn: raw::Instance) -> Self {
        Self { pfn }
    }
}
