use std::fmt::Debug;

pub(crate) struct InstanceDebug {
    pub(crate) v: openxr::Instance,
}

impl InstanceDebug {
    #[inline]
    pub(crate) fn fp(&self) -> &openxr::raw::Instance {
        self.v.fp()
    }
}

impl Debug for InstanceDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceDebug").finish()
    }
}

impl From<InstanceDebug> for openxr::Instance {
    fn from(item: InstanceDebug) -> Self {
        item.v
    }
}

impl From<openxr::Instance> for InstanceDebug {
    fn from(v: openxr::Instance) -> Self {
        Self { v }
    }
}
