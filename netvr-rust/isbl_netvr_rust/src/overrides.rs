use crate::instance::Instance;
use std::{collections::HashMap, os::raw::c_char, sync::RwLock};
use xr_layer::{log::LogError, sys, FnPtr};

struct Layer {
    pub(crate) layer: xr_layer::Layer,
    pub(crate) instances: HashMap<sys::Instance, Instance>,
}

impl Layer {
    pub(crate) fn new(func_in: sys::pfn::GetInstanceProcAddr) -> Self {
        let mut layer = unsafe { xr_layer::Layer::new(func_in) };
        layer
            .add_override(FnPtr::CreateInstance(create_instance))
            .add_override(FnPtr::CreateInstance(create_instance))
            .add_override(FnPtr::DestroyInstance(destroy_instance))
            .add_override(FnPtr::PollEvent(poll_event))
            .add_override(FnPtr::CreateActionSet(create_action_set))
            .add_override(FnPtr::CreateAction(create_action))
            .add_override(FnPtr::StringToPath(string_to_path))
            .add_override(FnPtr::SuggestInteractionProfileBindings(
                suggest_interaction_profile_bindings,
            ))
            .add_override(FnPtr::AttachSessionActionSets(attach_session_action_sets))
            .add_override(FnPtr::SyncActions(sync_actions))
            .add_override(FnPtr::GetActionStateBoolean(get_action_state_boolean))
            .add_override(FnPtr::GetActionStateFloat(get_action_state_float))
            .add_override(FnPtr::GetActionStateVector2f(get_action_state_vector2f))
            .add_override(FnPtr::GetActionStatePose(get_action_state_pose))
            .add_override(FnPtr::ApplyHapticFeedback(apply_haptic_feedback))
            .add_override(FnPtr::CreateSession(create_session))
            .add_override(FnPtr::DestroySession(destroy_session));
        Self {
            layer,
            instances: HashMap::default(),
        }
    }
}

lazy_static! {
    static ref LAYER: RwLock<Option<Layer>> = RwLock::new(None);
}

pub(crate) fn init(func_in: sys::pfn::GetInstanceProcAddr) -> sys::pfn::GetInstanceProcAddr {
    let layer = Layer::new(func_in);
    let mut w = LAYER.write().unwrap();
    *w = Some(layer);
    get_instance_proc_addr
}

pub(crate) fn deinit() {
    let mut w = LAYER.write().unwrap();
    *w = None;
}

extern "system" fn get_instance_proc_addr(
    instance: sys::Instance,
    name: *const c_char,
    function: *mut Option<sys::pfn::VoidFunction>,
) -> sys::Result {
    let r = LAYER.write().unwrap();

    match &*r {
        Some(layer) => match instance {
            sys::Instance::NULL => layer.layer.get_instance_proc_addr(instance, name, function),
            _ => {
                if layer.instances.contains_key(&instance) {
                    // we have the instance, use override
                    layer.layer.get_instance_proc_addr(instance, name, function)
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
        },
        None => sys::Result::ERROR_RUNTIME_FAILURE,
    }
}

extern "system" fn create_instance(
    create_info: *const sys::InstanceCreateInfo,
    instance_ptr: *mut sys::Instance,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn destroy_instance(instance_handle: sys::Instance) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn poll_event(
    instance_handle: sys::Instance,
    event_data: *mut sys::EventDataBuffer,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn create_action_set(
    instance_handle: sys::Instance,
    info: *const sys::ActionSetCreateInfo,
    action_set_ptr: *mut sys::ActionSet,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
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
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn suggest_interaction_profile_bindings(
    instance_handle: sys::Instance,
    suggested_bindings: *const sys::InteractionProfileSuggestedBinding,
) -> sys::Result {
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

extern "system" fn create_session(
    instance_handle: sys::Instance,
    create_info: *const sys::SessionCreateInfo,
    session_ptr: *mut sys::Session,
) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}

extern "system" fn destroy_session(session_handle: sys::Session) -> sys::Result {
    sys::Result::ERROR_RUNTIME_FAILURE
}
