use std::collections::HashMap;

use xr_layer::{safe_openxr, sys};

pub(crate) struct Instance {
    pub(crate) instance: safe_openxr::Instance,
    pub(crate) sessions: HashMap<sys::Session, safe_openxr::Session<safe_openxr::AnyGraphics>>,
}

impl Instance {
    pub(crate) fn new(instance: safe_openxr::Instance) -> Self {
        Self {
            instance,
            sessions: HashMap::default(),
        }
    }

    pub(crate) fn fp(&self) -> &xr_layer::raw::Instance {
        self.instance.fp()
    }
}
