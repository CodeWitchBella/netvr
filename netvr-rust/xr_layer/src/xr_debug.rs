use std::{borrow::BorrowMut, ffi::CStr, fmt};

use crate::{
    utils::ResultConvertible,
    xr_struct::{self, ActionCreateInfo},
    XrIterator,
};

pub struct XrDebugValue<'a, T: XrDebug>(pub(crate) openxr::Instance, pub(crate) &'a T);

impl<'a, T: XrDebug> fmt::Debug for XrDebugValue<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.xr_fmt(f, &self.0)
    }
}

/// Allows object to be debugged with extra information obtained from OpenXR
/// runtime.
pub trait XrDebug {
    /// Acts similarly to std::fmt::Debug::fmt but may call OpenXR function to
    /// reveal further detail about given object.
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result;

    /// This is usually how you consume object which implements XrDebug trait.
    ///
    /// If conversion to string and/or printing the object is desired
    /// ```
    /// format!("{:?}", object.as_debug(&self.instance))
    /// ```
    ///
    /// Or if you are implementing Debug or XrDebug (usually depending on the
    /// availability of openxr::Instance reference)
    ///
    /// ```
    /// f.field("field", &self.field.as_debug(instance))
    /// ```
    fn as_debug(&self, instance: &openxr::Instance) -> XrDebugValue<Self>
    where
        Self: std::marker::Sized + XrDebug,
    {
        XrDebugValue(instance.clone(), self)
    }
}

impl XrDebug for XrIterator {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let mut f = f.debug_list();
        for item in unsafe { self.unsafe_clone() } {
            f.entry(&item.as_debug(instance));
        }
        f.finish()
    }
}

impl<T> XrDebug for &T
where
    T: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        (*self).xr_fmt(f, instance)
    }
}

impl<T> XrDebug for Option<T>
where
    T: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        f.debug_struct("None").finish()
    }
}

impl XrDebug for ActionCreateInfo<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let mut f = f.debug_struct("ActionCreateInfo");
        let value = self.action_name();
        let f = f.field("action_name", &value);
        let value = self.action_type();
        let f = f.field("action_type", &value);
        let value: Vec<String> = self
            .subaction_paths()
            .map(|path| instance.path_to_string(path))
            .filter_map(Result::ok)
            .collect();
        let f = f.field("subaction_paths", &value);
        let value = self.localized_action_name();
        let f = f.field("localized_action_name", &value);
        f.finish()
    }
}

impl XrDebug for crate::xr_struct::EventDataSessionStateChanged<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let raw = self.as_raw();
        f.debug_struct("EventDataSessionStateChanged")
            .field("session", &raw.session)
            .field("state", &raw.state)
            .field("time", &XrDebugValue(instance.clone(), &raw.time))
            .finish()
    }
}

impl XrDebug for openxr_sys::Time {
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        let mut f = f.debug_struct("Time");
        let mut value = self.as_nanos();
        f.field("raw", &value);
        f.field("ns", &(value % 1000));
        value /= 1000;
        if value < 1 {
            return f.finish();
        }
        f.field("us", &(value % 1000));
        value /= 1000;
        if value < 1 {
            return f.finish();
        }
        f.field("ms", &(value % 1000));
        value /= 1000;
        if value < 1 {
            return f.finish();
        }
        f.field("s", &(value % 60));
        value /= 60;
        if value < 1 {
            return f.finish();
        }
        f.field("min", &(value % 60));
        value /= 60;
        if value < 1 {
            return f.finish();
        }
        f.field("h", &(value % 24));
        value /= 24;
        if value < 1 {
            return f.finish();
        }
        f.field("d", &value);
        f.finish()
    }
}

impl XrDebug for xr_struct::ActionsSyncInfo<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActionsSyncInfo")
            .field(
                "active_action_sets",
                &self.active_action_sets().as_debug(instance),
            )
            .finish()
    }
}

impl XrDebug for openxr_sys::ActiveActionSet {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActiveActionSet")
            .field("action_set", &self.action_set.as_debug(instance))
            .field("subaction_path", &self.subaction_path.as_debug(instance))
            .finish()
    }
}

/// Utility function which takes openxr_sys::Path and prints it with formatter.
/// If something goes wrong it returns empty error ().
fn debug_path(
    path: &openxr_sys::Path,
    f: &mut fmt::Formatter,
    instance: &openxr::Instance,
) -> Result<Result<(), fmt::Error>, ()> {
    let mut size_u32: u32 = 0;

    unsafe {
        (instance.fp().path_to_string)(
            instance.as_raw(),
            *path,
            0,
            size_u32.borrow_mut(),
            std::ptr::null_mut(),
        )
        .into_result()
        .map_err(|_| ())?;
        let size: usize = size_u32.try_into().map_err(|_| ())?;

        let mut vec = vec![0_u8; size];
        (instance.fp().path_to_string)(
            instance.as_raw(),
            *path,
            size_u32,
            size_u32.borrow_mut(),
            std::mem::transmute(vec.as_mut_ptr()),
        )
        .into_result()
        .map_err(|_| ())?;

        let str = CStr::from_bytes_with_nul(vec.as_slice())
            .map_err(|_| ())?
            .to_str()
            .map_err(|_| ())?;
        Ok(f.debug_tuple("Path").field(&str).finish())
    }
}

impl XrDebug for openxr_sys::Path {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        match debug_path(self, f, instance) {
            Ok(result) => result,
            Err(_) => f.debug_tuple("Path").field(&"<invalid>").finish(),
        }
    }
}

macro_rules! implement_as_non_exhaustive {
    ($($id: ty), *,) => {
        $(
            impl XrDebug for $id {
                fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
                    f.debug_struct(stringify!($id)).finish_non_exhaustive()
                }
            }
        )*
    };
}

implement_as_non_exhaustive!(
    openxr_sys::ActionSet,
    openxr_sys::ActionCreateInfo,
    openxr_sys::Action,
    openxr_sys::Session,
    openxr_sys::SessionState,
    xr_struct::EventDataInteractionProfileChanged<'_>,
    xr_struct::EventDataBuffer<'_>,
);
