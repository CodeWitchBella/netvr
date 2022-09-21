use crate::xr_structures::ActionCreateInfo;

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

impl<'a, T> XrDebug<'a> for Option<&'a T>
where
    T: XrDebug<'a>,
{
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
        match self {
            Some(v) => v.xr_debug(instance),
            None => XrDebugValue::new(instance.clone(), |_debuggable, f| {
                f.debug_struct("None").finish()
            }),
        }
    }
}

impl<'a> XrDebug<'a> for ActionCreateInfo<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
        XrDebugValue::new(instance.clone(), |debuggable, f| {
            let instance = &debuggable.instance;
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

impl<'a> XrDebug<'a> for crate::xr_structures::EventDataSessionStateChanged<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> crate::XrDebugValue<'a> {
        crate::XrDebugValue::new(instance.clone(), |_debuggable, f| {
            f.debug_tuple("EventDataSessionStateChanged")
                .field(&"<TODO>")
                .finish()
        })
    }
}

macro_rules! implement_as_hidden {
    ($($id: ident), *,) => {
        $(
            impl<'a> XrDebug<'a> for openxr_sys::$id {
                fn xr_debug(&'a self, instance: &openxr::Instance) -> XrDebugValue<'a> {
                    XrDebugValue::new(instance.clone(), |_debuggable, f| {
                        f.debug_tuple(stringify!($id)).field(&"<hidden>").finish()
                    })
                }
            }
        )*
    };
}

implement_as_hidden!(
    ActionSet,
    ActionCreateInfo,
    Action,
    Session,
    ActionsSyncInfo,
);
