use crate::{utils::ResultConvertible, UnsafeFrom, XrDebug, XrIterator, XrResult};
use std::fmt;

pub struct GetActionState {
    pub(crate) instance: openxr::Instance,
    pub(crate) session_handle: openxr_sys::Session,
    pub(crate) get_info: *const openxr_sys::ActionStateGetInfo,
}

/// Shared implementations between GetActionState*
impl GetActionState {
    pub fn get_info(&self) -> XrIterator {
        unsafe { XrIterator::from_ptr(self.get_info) }
    }

    fn partial_fmt<T: XrDebug>(
        &self,
        name: &str,
        f: &mut fmt::Formatter,
        state: *mut T,
    ) -> Result<(), std::fmt::Error> {
        f.debug_struct(name)
            .field("instance", &self.instance.as_raw())
            .field(
                "session_handle",
                &self.session_handle.as_debug(&self.instance),
            )
            .field("get_info", &self.get_info().as_debug(&self.instance))
            .field(
                "state",
                &unsafe { state.as_ref() }.map(|v| v.as_debug(&self.instance)),
            )
            .finish()
    }
}

macro_rules! implement {
    ($id: ident $xr: ident $fn: ident) => {
        pub struct $id {
            pub(crate) base: GetActionState,
            pub(crate) state: *mut openxr_sys::$xr,
        }

        impl $id {
            pub fn get(&self) -> XrResult<()> {
                unsafe {
                    (self.base.instance.fp().$fn)(
                        self.base.session_handle,
                        self.base.get_info,
                        self.state,
                    )
                }
                .into_result()
            }

            pub fn get_info(&self) -> XrIterator {
                self.base.get_info()
            }
        }

        impl std::fmt::Debug for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.base.partial_fmt(stringify!($id), f, self.state)
            }
        }
    };
}

implement!(GetActionStateBoolean ActionStateBoolean get_action_state_boolean);
implement!(GetActionStateFloat ActionStateFloat get_action_state_float);
implement!(GetActionStateVector2f ActionStateVector2f get_action_state_vector2f);
implement!(GetActionStatePose ActionStatePose get_action_state_pose);
