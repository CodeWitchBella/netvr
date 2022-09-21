use crate::{utils::ResultConvertible, xr_structures::XrIterator, XrDebug, XrResult};

pub struct CreateAction {
    pub(crate) instance: openxr::Instance,
    pub(crate) action_set_handle: openxr_sys::ActionSet,
    pub(crate) info: *const openxr_sys::ActionCreateInfo,
    pub(crate) out: *mut openxr_sys::Action,
}

impl CreateAction {
    pub fn create_action(&self) -> XrResult<()> {
        unsafe { (self.instance.fp().create_action)(self.action_set_handle, self.info, self.out) }
            .into_result()
    }

    pub fn instance(&self) -> &openxr::Instance {
        &self.instance
    }

    pub fn info(&self) -> XrIterator {
        self.info.into()
    }
}

impl std::fmt::Debug for CreateAction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("CreateAction")
            .field("instance", &self.instance.as_raw())
            .field(
                "action_set",
                &self.action_set_handle.xr_debug(&self.instance),
            )
            .field(
                "info",
                &unsafe { self.info.read() }.xr_debug(&self.instance),
            )
            .field(
                "out",
                &unsafe { self.out.as_ref() }.xr_debug(&self.instance),
            )
            .finish()
    }
}
