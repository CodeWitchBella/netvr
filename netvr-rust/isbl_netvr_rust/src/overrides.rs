use crate::{
    implementation::post_sync_actions,
    instance::Instance,
    xr_wrap::{xr_wrap_trace, ResultConvertible, XrWrapError},
};
use std::{collections::HashMap, os::raw::c_char, sync::RwLock};
use xr_layer::{
    log::{LogError, LogInfo, LogTrace, LogWarn},
    safe_openxr::{self, InstanceExtensions},
    sys, Entry, FnPtr, SessionCreateInfo, UnsafeFrom, XrDebug, XrIterator,
};

struct Layer {
    pub(self) layer: xr_layer::Layer,
    /// Entry is one of the few structs from openxr crate which do not implement
    /// drop and is therefore safe to use.
    pub(self) entry: Entry,
    pub(self) instances: HashMap<sys::Instance, Instance>,
    pub(self) sessions: HashMap<sys::Session, sys::Instance>,
    pub(self) action_sets: HashMap<sys::ActionSet, sys::Instance>,
}

impl Layer {
    /// Sets up all required plumbing to be able to run the layer
    pub(self) fn new(func_in: sys::pfn::GetInstanceProcAddr, manual_unhook: bool) -> Self {
        let mut layer = unsafe { xr_layer::Layer::new(func_in) };
        layer
            .add_override(FnPtr::CreateInstance(create_instance))
            .add_override(FnPtr::PollEvent(poll_event))
            .add_override(FnPtr::CreateActionSet(create_action_set))
            .add_override(FnPtr::CreateAction(create_action))
            .add_override(FnPtr::StringToPath(string_to_path))
            .add_override(FnPtr::SuggestInteractionProfileBindings(
                suggest_interaction_profile_bindings,
            ))
            .add_override(FnPtr::CreateSession(create_session))
            .add_override(FnPtr::DestroySession(destroy_session))
            .add_override(FnPtr::AttachSessionActionSets(attach_session_action_sets))
            .add_override(FnPtr::SyncActions(sync_actions))
            .add_override(FnPtr::GetActionStateBoolean(get_action_state_boolean))
            .add_override(FnPtr::GetActionStateFloat(get_action_state_float))
            .add_override(FnPtr::GetActionStateVector2f(get_action_state_vector2f))
            .add_override(FnPtr::GetActionStatePose(get_action_state_pose))
            .add_override(FnPtr::ApplyHapticFeedback(apply_haptic_feedback))
            .add_override(FnPtr::LocateViews(locate_views));
        if !manual_unhook {
            layer.add_override(FnPtr::DestroyInstance(destroy_instance));
        }
        let entry = unsafe { Entry::from_get_instance_proc_addr(func_in) }.unwrap();
        Self {
            layer,
            entry,
            instances: HashMap::default(),
            sessions: HashMap::default(),
            action_sets: HashMap::default(),
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
    xr_wrap_trace("create_instance", || {
        let mut w = LAYER.write()?;
        let layer = (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
        let result = unsafe { (layer.entry.fp().create_instance)(create_info, instance_ptr) };
        if result == sys::Result::SUCCESS {
            let instance_handle = unsafe { *instance_ptr };
            let instance_result = unsafe {
                safe_openxr::Instance::from_raw(
                    layer.entry.clone(),
                    instance_handle,
                    InstanceExtensions::default(),
                )
            };
            if let Ok(instance) = instance_result {
                layer
                    .instances
                    .insert(instance_handle, Instance::new(instance));
            } else {
                LogWarn::str("Failed to acquire Instance");
            }
        }
        result.into_result()
    })
}

extern "system" fn destroy_instance(instance_handle: sys::Instance) -> sys::Result {
    xr_wrap_trace("destroy_instance", || {
        let mut w = LAYER.write()?;
        let layer = (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
        let instance = layer
            .instances
            .remove(&instance_handle)
            .ok_or(sys::Result::ERROR_INSTANCE_LOST)?;
        // This is already done by above's drop:
        //let result = unsafe { (instance.fp().destroy_instance)(instance_handle) };
        //result.into_result()
        Ok(())
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

fn read_layer_mut<'a>(
    w: &'a mut std::sync::RwLockWriteGuard<Option<Layer>>,
) -> Result<&'a mut Layer, xr_layer::sys::Result> {
    (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)
}

fn read_instance_layer_mut(
    instances: &mut HashMap<sys::Instance, Instance>,
    instance_handle: sys::Instance,
) -> Result<&mut Instance, xr_layer::sys::Result> {
    instances
        .get_mut(&instance_handle)
        .ok_or(sys::Result::ERROR_INSTANCE_LOST)
}

fn subresource_read_instance<'a, T: std::hash::Hash + std::cmp::Eq>(
    r: &'a std::sync::RwLockReadGuard<Option<Layer>>,
    reader: fn(layer: &Layer) -> &HashMap<T, sys::Instance>,
    handle: T,
) -> Result<&'a Instance, xr_layer::sys::Result> {
    let layer = (*r).as_ref().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
    let instance_handle = reader(layer)
        .get(&handle)
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
    layer
        .instances
        .get(instance_handle)
        .ok_or(sys::Result::ERROR_INSTANCE_LOST)
}

fn subresource_delete<'a, T: std::hash::Hash + std::cmp::Eq>(
    w: &'a mut std::sync::RwLockWriteGuard<Option<Layer>>,
    reader: fn(layer: &mut Layer) -> &mut HashMap<T, sys::Instance>,
    handle: T,
) -> Result<&'a mut Instance, xr_layer::sys::Result> {
    // TODO: deleting instance should also delete sub-resources, and eg. deleting
    // ActionSet should also delete Action, etc.

    let layer = (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
    let instance_handle = reader(layer)
        .remove(&handle)
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
    layer
        .instances
        .get_mut(&instance_handle)
        .ok_or(sys::Result::ERROR_INSTANCE_LOST)
}

extern "system" fn poll_event(
    instance_handle: sys::Instance,
    event_data: *mut sys::EventDataBuffer,
) -> sys::Result {
    xr_wrap_trace("poll_event", || {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result = unsafe { (instance.fp().poll_event)(instance_handle, event_data) };
        result.into_result()
    })
}

extern "system" fn create_action_set(
    instance_handle: sys::Instance,
    info: *const sys::ActionSetCreateInfo,
    action_set_ptr: *mut sys::ActionSet,
) -> sys::Result {
    xr_wrap_trace("create_action_set", || {
        let mut w = LAYER.write()?;
        let layer = read_layer_mut(&mut w)?;
        let instance = read_instance_layer_mut(&mut layer.instances, instance_handle)?;

        let result =
            unsafe { (instance.fp().create_action_set)(instance_handle, info, action_set_ptr) };
        if result.into_result().is_ok() {
            let action_set = unsafe { *action_set_ptr };
            layer.action_sets.insert(action_set, instance_handle);
        }
        result.into_result()
    })
}

extern "system" fn create_action(
    action_set_handle: sys::ActionSet,
    info: *const sys::ActionCreateInfo,
    out: *mut sys::Action,
) -> sys::Result {
    xr_wrap_trace("create_action", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.action_sets, action_set_handle)?;
        let result = unsafe { (instance.fp().create_action)(action_set_handle, info, out) };
        result.into_result()
    })
}

extern "system" fn string_to_path(
    instance_handle: sys::Instance,
    path_string_raw: *const c_char,
    path: *mut sys::Path,
) -> sys::Result {
    xr_wrap_trace("string_to_path", || {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result =
            unsafe { (instance.fp().string_to_path)(instance_handle, path_string_raw, path) };
        result.into_result()
    })
}

extern "system" fn suggest_interaction_profile_bindings(
    instance_handle: sys::Instance,
    suggested_bindings: *const sys::InteractionProfileSuggestedBinding,
) -> sys::Result {
    xr_wrap_trace("suggest_interaction_profile_bindings", || {
        let r = LAYER.read()?;
        let instance = read_instance(&r, instance_handle)?;

        let result = unsafe {
            (instance.fp().suggest_interaction_profile_bindings)(
                instance_handle,
                suggested_bindings,
            )
        };
        result.into_result()
    })
}

extern "system" fn create_session(
    instance_handle: sys::Instance,
    create_info_ptr: *const sys::SessionCreateInfo,
    session_ptr: *mut sys::Session,
) -> sys::Result {
    xr_wrap_trace("create_session", || {
        let mut w = LAYER.write()?;
        let layer = read_layer_mut(&mut w)?;
        let instance = read_instance_layer_mut(&mut layer.instances, instance_handle)?;

        let result = unsafe {
            (instance.fp().create_session)(instance_handle, create_info_ptr, session_ptr)
        };
        if result.into_result().is_ok() {
            let session_handle = unsafe { *session_ptr };
            layer.sessions.insert(session_handle, instance_handle);
            let session = unsafe {
                // Note: this is slightly wrong, but works unless we use graphics
                // api. We do not do that and do immediately convert it to AnyGraphics.
                safe_openxr::Session::<safe_openxr::Vulkan>::from_raw(
                    instance.instance.clone(),
                    session_handle,
                    Box::new(()),
                )
            }
            .0
            .into_any_graphics();

            instance.sessions.insert(session_handle, session);
        }
        result.into_result()
    })
}

extern "system" fn destroy_session(session_handle: sys::Session) -> sys::Result {
    xr_wrap_trace("destroy_session", || {
        let mut w = LAYER.write()?;
        let instance = subresource_delete(&mut w, |l| &mut l.sessions, session_handle)?;

        instance
            .sessions
            .remove(&session_handle)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
        Ok(())
        // already handled:
        //let result = unsafe { (instance.fp().destroy_session)(session_handle) };
        //result.into_result()
    })
}

extern "system" fn attach_session_action_sets(
    session_handle: sys::Session,
    attach_info: *const sys::SessionActionSetsAttachInfo,
) -> sys::Result {
    xr_wrap_trace("attach_session_action_sets", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result =
            unsafe { (instance.fp().attach_session_action_sets)(session_handle, attach_info) };
        result.into_result()
    })
}

extern "system" fn sync_actions(
    session_handle: sys::Session,
    sync_info: *const sys::ActionsSyncInfo,
) -> sys::Result {
    xr_wrap_trace("sync_actions", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result = unsafe { (instance.fp().sync_actions)(session_handle, sync_info) };
        post_sync_actions(instance, unsafe { XrIterator::from_ptr(sync_info) });
        result.into_result()
    })
}

extern "system" fn get_action_state_boolean(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateBoolean,
) -> sys::Result {
    xr_wrap_trace("get_action_state_boolean", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result =
            unsafe { (instance.fp().get_action_state_boolean)(session_handle, get_info, state) };
        result.into_result()
    })
}

extern "system" fn get_action_state_float(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateFloat,
) -> sys::Result {
    xr_wrap_trace("get_action_state_float", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result =
            unsafe { (instance.fp().get_action_state_float)(session_handle, get_info, state) };
        result.into_result()
    })
}

extern "system" fn get_action_state_vector2f(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateVector2f,
) -> sys::Result {
    xr_wrap_trace("get_action_state_vector2f", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result =
            unsafe { (instance.fp().get_action_state_vector2f)(session_handle, get_info, state) };
        result.into_result()
    })
}

extern "system" fn get_action_state_pose(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStatePose,
) -> sys::Result {
    xr_wrap_trace("get_action_state_pose", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result =
            unsafe { (instance.fp().get_action_state_pose)(session_handle, get_info, state) };
        result.into_result()
    })
}

extern "system" fn apply_haptic_feedback(
    session_handle: sys::Session,
    haptic_action_info: *const sys::HapticActionInfo,
    haptic_feedback: *const sys::HapticBaseHeader,
) -> sys::Result {
    xr_wrap_trace("apply_haptic_feedback", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result = unsafe {
            (instance.fp().apply_haptic_feedback)(
                session_handle,
                haptic_action_info,
                haptic_feedback,
            )
        };
        result.into_result()
    })
}

extern "system" fn locate_views(
    session_handle: sys::Session,
    view_locate_info: *const sys::ViewLocateInfo,
    view_state: *mut sys::ViewState,
    view_capacity_input: u32,
    view_count_output: *mut u32,
    view: *mut sys::View,
) -> sys::Result {
    xr_wrap_trace("locate_views", || {
        let r = LAYER.read()?;
        let instance = subresource_read_instance(&r, |l| &l.sessions, session_handle)?;

        let result = unsafe {
            (instance.fp().locate_views)(
                session_handle,
                view_locate_info,
                view_state,
                view_capacity_input,
                view_count_output,
                view,
            )
        };
        result.into_result()
    })
}

pub(crate) fn with_layer<T>(handle: sys::Instance, cb: T) -> Result<(), XrWrapError>
where
    T: FnOnce(&Instance) -> Result<(), XrWrapError>,
{
    let r = LAYER.read()?;
    let instance = read_instance(&r, handle)?;
    cb(instance)
}
