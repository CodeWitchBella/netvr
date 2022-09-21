use crate::xr_structures::ActionCreateInfo;

pub trait Debuggable<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> DebuggableValue<'a>;
}

pub struct DebuggableValue<'a> {
    pub(crate) fun:
        Box<dyn Fn(&DebuggableValue, &mut std::fmt::Formatter) -> std::fmt::Result + 'a>,
    pub(crate) instance: openxr::Instance,
}

impl std::fmt::Debug for DebuggableValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.fun.as_ref()(self, f)
    }
}

impl<'a> DebuggableValue<'a> {
    pub(crate) fn new<
        T: Fn(&DebuggableValue, &mut std::fmt::Formatter) -> std::fmt::Result + 'a,
    >(
        instance: openxr::Instance,
        fun: T,
    ) -> Self {
        Self {
            fun: Box::new(fun),
            instance,
        }
    }
}

impl<'a> Debuggable<'a> for ActionCreateInfo<'a> {
    fn xr_debug(&'a self, instance: &openxr::Instance) -> DebuggableValue<'a> {
        DebuggableValue::new(instance.clone(), |debuggable, f| {
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
