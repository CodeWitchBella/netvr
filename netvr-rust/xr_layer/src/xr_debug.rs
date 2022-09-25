use std::fmt;

use crate::{
    xr_struct::{self, ActionCreateInfo},
    XrIterator,
};

pub struct XrDebugValue<'a, T: XrDebug>(pub(crate) openxr::Instance, pub(crate) &'a T);

impl<'a, T: XrDebug> fmt::Debug for XrDebugValue<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.xr_fmt(f, &self.0)
    }
}

pub trait XrDebug {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result;

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
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActionsSyncInfo")
            .field("active_action_sets", &self.active_action_sets())
            .finish()
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

pub(crate) struct DebugFn<T>
where
    T: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    fun: T,
}

impl<T> std::fmt::Debug for DebugFn<T>
where
    T: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self.fun)(f)
    }
}

impl<T> DebugFn<T>
where
    T: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    pub(crate) fn new(fun: T) -> Self {
        Self { fun }
    }
}
