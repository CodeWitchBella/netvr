use crate::{instance::Instance, xr_wrap::xr_wrap, xr_wrap::ResultConvertible};
use std::{collections::HashMap, os::raw::c_char, sync::RwLock};
use xr_layer::{
    log::{LogError, LogTrace, LogWarn},
    raw, sys, Entry, FnPtr,
};

struct Layer {
    pub(self) layer: xr_layer::Layer,
    /// Entry is one of the few structs from openxr crate which do not implement
    /// drop and is therefore safe to use.
    pub(self) entry: Entry,
    pub(self) instances: HashMap<sys::Instance, Instance>,
}

impl Layer {
    /// Sets up all required plumbing to be able to run the layer
    pub(self) fn new(func_in: sys::pfn::GetInstanceProcAddr, manual_unhook: bool) -> Self {
        let mut layer = unsafe { xr_layer::Layer::new(func_in) };
        layer
            .add_override(FnPtr::CreateInstance(create_instance))
            .add_override(FnPtr::PollEvent(poll_event))
            .add_override(FnPtr::CreateActionSet(create_action_set))
            //.add_override(FnPtr::CreateAction(create_action))
            .add_override(FnPtr::StringToPath(string_to_path))
            .add_override(FnPtr::SuggestInteractionProfileBindings(
                suggest_interaction_profile_bindings,
            ))
            .add_override(FnPtr::CreateSession(create_session))
            //.add_override(FnPtr::DestroySession(destroy_session))
            //.add_override(FnPtr::AttachSessionActionSets(attach_session_action_sets))
            //.add_override(FnPtr::SyncActions(sync_actions))
            //.add_override(FnPtr::GetActionStateBoolean(get_action_state_boolean))
            //.add_override(FnPtr::GetActionStateFloat(get_action_state_float))
            //.add_override(FnPtr::GetActionStateVector2f(get_action_state_vector2f))
            //.add_override(FnPtr::GetActionStatePose(get_action_state_pose))
            //.add_override(FnPtr::ApplyHapticFeedback(apply_haptic_feedback))
          ;
        if !manual_unhook {
            layer.add_override(FnPtr::DestroyInstance(destroy_instance));
        }
        let entry = unsafe { Entry::from_get_instance_proc_addr(func_in) }.unwrap();
        Self {
            layer,
            entry,
            instances: HashMap::default(),
        }
    }
}

lazy_static! {
    static ref LAYER: RwLock<Option<Layer>> = RwLock::new(None);
}

/// Implementation of main entrypoint to the layer. This function sets up global
/// reference which is required for function pointers to work.
/// Then it returns function pointer which you are supposed to call instead of
/// xrGetInstanceProcAddr you sent in.
pub(crate) fn init(
    func_in: sys::pfn::GetInstanceProcAddr,
    manual_unhook: bool,
) -> sys::pfn::GetInstanceProcAddr {
    LogTrace::str("init");
    let layer = Layer::new(func_in, manual_unhook);
    let mut w = LAYER.write().unwrap();
    *w = Some(layer);
    get_instance_proc_addr
}

/// Removes all global data. It is up to you to make sure that you do not call
/// any function you got from `xrGetInstanceProcAddr` after this point.
///
/// It might crash, or it might return data from new instantiation. ✨Undefined
/// behaviour✨ is undefined. In other words: you will have a bad time debugging
/// this mess if you mess this up.
pub(crate) fn deinit() {
    LogTrace::str("deinit");
    let mut w = LAYER.write().unwrap();
    *w = None;
}

extern "system" fn get_instance_proc_addr(
    instance: sys::Instance,
    name: *const c_char,
    function: *mut Option<sys::pfn::VoidFunction>,
) -> sys::Result {
    let r = LAYER.read().unwrap();

    match &*r {
        Some(layer) => {
            let find_override = || match layer.layer.get_instance_proc_addr(instance, name) {
                Ok(fun) => {
                    unsafe { *function = fun };
                    sys::Result::SUCCESS
                }
                Err(error) => {
                    unsafe { *function = None };
                    error
                }
            };
            match instance {
                sys::Instance::NULL => find_override(),
                _ => {
                    if layer.instances.contains_key(&instance) {
                        find_override()
                    } else {
                        LogError::str(
                            "get_instance_proc_addr: Lost instance, using runtime's implementation",
                        );
                        unsafe {
                            layer
                                .layer
                                .get_instance_proc_addr_runtime(instance, name, function)
                        }
                    }
                }
            }
        }
        None => sys::Result::ERROR_RUNTIME_FAILURE,
    }
}

extern "system" fn create_instance(
    create_info: *const sys::InstanceCreateInfo,
    instance_ptr: *mut sys::Instance,
) -> sys::Result {
    xr_wrap(|| {
        let mut w = LAYER.write()?;
        let layer = (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
        let result = unsafe { (layer.entry.fp().create_instance)(create_info, instance_ptr) };
        if result == sys::Result::SUCCESS {
            let instance_handle = unsafe { *instance_ptr };
            let raw_instance_result = unsafe { raw::Instance::load(&layer.entry, instance_handle) };
            if let Ok(raw_instance) = raw_instance_result {
                layer
                    .instances
                    .insert(instance_handle, Instance::new(raw_instance));
            } else {
                LogWarn::str("Failed to acquire raw::Instance");
            }
        }
        LogTrace::string(format!(
            "create_instance {:?} -> {:?}",
            create_info, instance_ptr
        ));
        result.into_result()
    })
}

extern "system" fn destroy_instance(instance_handle: sys::Instance) -> sys::Result {
    xr_wrap(|| {
        let mut w = LAYER.write()?;
        let layer = (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
        let instance = layer
            .instances
            .remove(&instance_handle)
            .ok_or(sys::Result::ERROR_INSTANCE_LOST)?;
        let result = unsafe { (instance.pfn.destroy_instance)(instance_handle) };
        LogTrace::string(format!(
            "destroy_instance {:?} -> {:?}",
            instance_handle, result
        ));
        result.into_result()
    })
}

fn read_instance<'a>(
    r: &'a std::sync::RwLockReadGuard<Option<Layer>>,
    instance_handle: sys::Instance,
) -> Result<&'a Instance, xr_layer::sys::Result> {
    let layer = (*r).as_ref().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
    layer
        .instances
        .get(&instance_handle)
        .ok_or(sys::Result::ERROR_INSTANCE_LOST)
}

extern "system" fn poll_event(
    instance_handle: sys::Instance,
    event_data: *mut sys::EventDataBuffer,
) -> sys::Result {
    xr_wrap(|| {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result = unsafe { (instance.pfn.poll_event)(instance_handle, event_data) };
        LogTrace::string(format!("poll_event {:?} -> {:?}", instance_handle, result));
        result.into_result()
    })
}

extern "system" fn create_action_set(
    instance_handle: sys::Instance,
    info: *const sys::ActionSetCreateInfo,
    action_set_ptr: *mut sys::ActionSet,
) -> sys::Result {
    xr_wrap(|| {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result =
            unsafe { (instance.pfn.create_action_set)(instance_handle, info, action_set_ptr) };
        LogTrace::string(format!(
            "create_action_set {:?} -> {:?}",
            instance_handle, result
        ));
        result.into_result()
    })
}

extern "system" fn create_action(
    action_set_handle: sys::ActionSet,
    info: *const sys::ActionCreateInfo,
    out: *mut sys::Action,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn string_to_path(
    instance_handle: sys::Instance,
    path_string_raw: *const c_char,
    path: *mut sys::Path,
) -> sys::Result {
    xr_wrap(|| {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result =
            unsafe { (instance.pfn.string_to_path)(instance_handle, path_string_raw, path) };
        LogTrace::string(format!(
            "string_to_path {:?} -> {:?}",
            instance_handle, result
        ));
        result.into_result()
    })
}

extern "system" fn suggest_interaction_profile_bindings(
    instance_handle: sys::Instance,
    suggested_bindings: *const sys::InteractionProfileSuggestedBinding,
) -> sys::Result {
    xr_wrap(|| {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result = unsafe {
            (instance.pfn.suggest_interaction_profile_bindings)(instance_handle, suggested_bindings)
        };
        LogTrace::string(format!(
            "suggest_interaction_profile_bindings {:?} -> {:?}",
            instance_handle, result
        ));
        result.into_result()
    })
}

extern "system" fn create_session(
    instance_handle: sys::Instance,
    create_info: *const sys::SessionCreateInfo,
    session_ptr: *mut sys::Session,
) -> sys::Result {
    xr_wrap(|| {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result =
            unsafe { (instance.pfn.create_session)(instance_handle, create_info, session_ptr) };
        LogTrace::string(format!(
            "create_session {:?} -> {:?}",
            instance_handle, result
        ));
        result.into_result()
    })
}

extern "system" fn destroy_session(session_handle: sys::Session) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn attach_session_action_sets(
    session_handle: sys::Session,
    attach_info: *const sys::SessionActionSetsAttachInfo,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn sync_actions(
    session_handle: sys::Session,
    sync_info: *const sys::ActionsSyncInfo,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn get_action_state_boolean(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateBoolean,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn get_action_state_float(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateFloat,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn get_action_state_vector2f(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateVector2f,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn get_action_state_pose(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStatePose,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn apply_haptic_feedback(
    session_handle: sys::Session,
    haptic_action_info: *const sys::HapticActionInfo,
    haptic_feedback: *const sys::HapticBaseHeader,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}
