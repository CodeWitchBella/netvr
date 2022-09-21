use crate::xr_struct::{self, ActionCreateInfo};

pub trait XrDebug<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a>;
}

pub struct XrDebugValue<'a> {
    pub(crate) fun: Box<dyn Fn(&XrDebugValue, &mut std::fmt::Formatter) -> std::fmt::Result + 'a>,
    pub(crate) instance: openxr::Instance,
}

impl std::fmt::Debug for XrDebugValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.fun.as_ref()(self, f)
    }
}

impl<'a> XrDebugValue<'a> {
    pub(crate) fn new<T: Fn(&XrDebugValue, &mut std::fmt::Formatter) -> std::fmt::Result + 'a>(
        instance: openxr::Instance,
        fun: T,
    ) -> Self {
        Self {
            fun: Box::new(fun),
            instance,
        }
    }
}

impl<'a, T> XrDebug<'a> for &'a T
where
    T: XrDebug<'a>,
{
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
        (*self).xr_debug(instance)
    }
}

impl<'a, T> XrDebug<'a> for Option<T>
where
    T: XrDebug<'a>,
{
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
        match self {
            Some(v) => v.xr_debug(instance),
            None => XrDebugValue::new(instance.clone(), |_debugable, f| {
                f.debug_struct("None").finish()
            }),
        }
    }
}

impl<'a> XrDebug<'a> for ActionCreateInfo<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
        XrDebugValue::new(instance.clone(), |debugable, f| {
            let instance = &debugable.instance;
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
        })
    }
}

impl<'a> XrDebug<'a> for crate::xr_struct::EventDataSessionStateChanged<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> crate::XrDebugValue<'a> {
        crate::XrDebugValue::new(instance.clone(), |debugable, f| {
            let raw = self.as_raw();
            f.debug_struct("EventDataSessionStateChanged")
                .field("session", &raw.session)
                .field("state", &raw.state)
                .field("time", &raw.time.xr_debug(&debugable.instance))
                .finish()
        })
    }
}

impl<'a> XrDebug<'a> for openxr_sys::Time {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> crate::XrDebugValue<'a> {
        crate::XrDebugValue::new(instance.clone(), |_debugable, f| {
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
        })
    }
}

impl<'a> XrDebug<'a> for xr_struct::ActionsSyncInfo<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
        XrDebugValue::new(instance.clone(), |_debugable, f| {
            f.debug_struct("ActionsSyncInfo")
                .field("active_action_sets", &self.active_action_sets())
                .finish()
        })
    }
}

macro_rules! implement_as_non_exhaustive {
    ($($id: ty), *,) => {
        $(
            impl<'a> XrDebug<'a> for $id {
                fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
                    XrDebugValue::new(instance.clone(), |_debugable, f| {
                        f.debug_struct(stringify!($id)).finish_non_exhaustive()
                    })
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
