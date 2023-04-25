use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr,
    sync::RwLock,
};

use anyhow::anyhow;
use netvr_data::net;
use tracing::{field, info, trace_span};
use xr_layer::{
    log::{LogError, LogInfo, LogTrace, LogWarn},
    raw,
    safe_openxr::{self, InstanceExtensions},
    sys::{self, ReferenceSpaceType},
    Entry, FnPtr, UnsafeFrom, XrDebug, XrStructChain,
};

use crate::{
    instance::{Action, ActionSet, Instance, Session},
    local_configuration::{self, InteractionProfile, LocalConfigurationSnapshot},
    xr_wrap::{xr_wrap, RecordDebug, ResultConvertible, Trace, XrWrapError},
};

#[derive(Default)]
struct InstanceRefs {
    pub(crate) sessions: HashMap<sys::Session, sys::Instance>,
    pub(crate) action_sets: HashMap<sys::ActionSet, sys::Instance>,
    pub(crate) spaces: HashMap<sys::Space, sys::Instance>,
    pub(crate) space_sessions: HashMap<sys::Space, sys::Session>,
}

struct Layer {
    pub(self) layer: xr_layer::Layer,
    /// Entry is one of the few structs from openxr crate which do not implement
    /// drop and is therefore safe to use.
    pub(self) entry: Entry,
    pub(self) instances: HashMap<sys::Instance, Instance>,
    pub(self) instance_refs: InstanceRefs,
    pub(self) trace: Trace,
}

impl Layer {
    /// Sets up all required plumbing to be able to run the layer
    pub(self) fn new(func_in: sys::pfn::GetInstanceProcAddr, manual_unhook: bool) -> Self {
        let mut layer = unsafe { xr_layer::Layer::new(func_in) };
        #[rustfmt::skip]
        layer
            .add_override(FnPtr::ApplyHapticFeedback(apply_haptic_feedback))
            .add_override(FnPtr::AttachSessionActionSets(attach_session_action_sets))
            .add_override(FnPtr::BeginFrame(begin_frame))
            .add_override(FnPtr::BeginSession(begin_session))
            .add_override(FnPtr::CreateAction(create_action))
            .add_override(FnPtr::CreateActionSet(create_action_set))
            .add_override(FnPtr::CreateActionSpace(create_action_space))
            .add_override(FnPtr::CreateInstance(create_instance))
            .add_override(FnPtr::CreateReferenceSpace(create_reference_space))
            .add_override(FnPtr::CreateSession(create_session))
            .add_override(FnPtr::DestroySession(destroy_session))
            .add_override(FnPtr::EndFrame(end_frame))
            .add_override(FnPtr::EndSession(end_session))
            .add_override(FnPtr::EnumerateBoundSourcesForAction(enumerate_bound_sources_for_action))
            .add_override(FnPtr::EnumerateEnvironmentBlendModes(enumerate_environment_blend_modes))
            .add_override(FnPtr::EnumerateReferenceSpaces(enumerate_reference_spaces))
            .add_override(FnPtr::EnumerateViewConfigurations(enumerate_view_configurations))
            .add_override(FnPtr::EnumerateViewConfigurationViews(enumerate_view_configuration_views))
            .add_override(FnPtr::GetActionStateBoolean(get_action_state_boolean))
            .add_override(FnPtr::GetActionStateFloat(get_action_state_float))
            .add_override(FnPtr::GetActionStatePose(get_action_state_pose))
            .add_override(FnPtr::GetActionStateVector2f(get_action_state_vector2f))
            .add_override(FnPtr::GetCurrentInteractionProfile(get_current_interaction_profile))
            .add_override(FnPtr::GetInputSourceLocalizedName(get_input_source_localized_name))
            .add_override(FnPtr::GetInstanceProperties(get_instance_properties))
            .add_override(FnPtr::GetReferenceSpaceBoundsRect(get_reference_space_bounds_rect))
            .add_override(FnPtr::GetSystem(get_system))
            .add_override(FnPtr::GetSystemProperties(get_system_properties))
            .add_override(FnPtr::GetViewConfigurationProperties(get_view_configuration_properties))
            .add_override(FnPtr::LocateViews(locate_views))
            .add_override(FnPtr::PathToString(path_to_string))
            .add_override(FnPtr::PollEvent(poll_event))
            .add_override(FnPtr::RequestExitSession(request_exit_session))
            .add_override(FnPtr::ResultToString(result_to_string))
            .add_override(FnPtr::StopHapticFeedback(stop_haptic_feedback))
            .add_override(FnPtr::StringToPath(string_to_path))
            .add_override(FnPtr::StructureTypeToString(structure_type_to_string))
            .add_override(FnPtr::SuggestInteractionProfileBindings(suggest_interaction_profile_bindings))
            .add_override(FnPtr::SyncActions(sync_actions))
            .add_override(FnPtr::WaitFrame(wait_frame))
            // Maybe TODO?:
            //.add_override(FnPtr::DestroySpace(destroy_space))
            //.add_override(FnPtr::DestroyAction(destroy_action))
            .add_override(FnPtr::DestroyActionSet(destroy_action_set))
            //.add_override(FnPtr::EnumerateApiLayerProperties(enumerate_api_layer_properties))
            //.add_override(FnPtr::EnumerateInstanceExtensionProperties(enumerate_instance_extension_properties))
            .add_override(FnPtr::LocateSpace(locate_space))
            // Swapchain-related functions are of no interest to me
            //.add_override(FnPtr::AcquireSwapchainImage(acquire_swapchain_image))
            //.add_override(FnPtr::CreateSwapchain(create_swapchain))
            //.add_override(FnPtr::DestroySwapchain(destroy_swapchain))
            //.add_override(FnPtr::EnumerateSwapchainFormats(enumerate_swapchain_formats))
            //.add_override(FnPtr::EnumerateSwapchainImages(enumerate_swapchain_images))
            //.add_override(FnPtr::ReleaseSwapchainImage(release_swapchain_image))
            //.add_override(FnPtr::WaitSwapchainImage(wait_swapchain_image))
            ;
        if !manual_unhook {
            layer.add_override(FnPtr::DestroyInstance(destroy_instance));
        }
        let entry = unsafe { Entry::from_get_instance_proc_addr(func_in) }.unwrap();
        Self {
            layer,
            entry,
            instances: HashMap::default(),
            instance_refs: Default::default(),
            trace: Trace::new(),
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
    xr_wrap(|| {
        LogTrace::str("init");
        let layer = Layer::new(func_in, manual_unhook);
        let mut w = LAYER.write().unwrap();
        *w = Some(layer);
        Ok(())
    });
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

fn wrap<O>(f: O) -> sys::Result
where
    O: FnOnce(&Layer) -> Result<(), XrWrapError>,
    O: std::panic::UnwindSafe,
{
    xr_wrap(|| {
        let r = LAYER.read()?;
        let layer = (*r).as_ref().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
        layer.trace.wrap(|| f(layer))
    })
}

fn wrap_mut<O>(function: O) -> sys::Result
where
    O: FnOnce(&mut Layer) -> Result<(), XrWrapError>,
    O: std::panic::UnwindSafe,
{
    xr_wrap(|| {
        let mut w = LAYER.write()?;
        let layer = (*w).as_mut().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
        layer.trace.clone().wrap(|| function(layer))
    })
}

extern "system" fn get_instance_proc_addr(
    instance: sys::Instance,
    name: *const c_char,
    function: *mut Option<sys::pfn::VoidFunction>,
) -> sys::Result {
    wrap(|layer| {
        let find_override = || match layer.layer.get_instance_proc_addr(instance, name) {
            Ok(fun) => {
                unsafe { *function = fun };
                Ok(())
            }
            Err(error) => {
                unsafe { *function = None };
                error.into_result()
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
                    .into_result()
                }
            }
        }
    })
}

extern "system" fn create_instance(
    create_info_ptr: *const sys::InstanceCreateInfo,
    instance_ptr: *mut sys::Instance,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("create_instance").entered();
        LogInfo::str("create_instance");

        let timespec = CString::new("XR_KHR_convert_timespec_time")?;
        let mut has_timespec = false;
        #[cfg(windows)]
        let perf_counter = CString::new("XR_KHR_win32_convert_performance_counter_time")?;
        #[cfg(windows)]
        let mut has_perf_counter = false;

        let mut extensions = vec![];
        let mut create_info = unsafe { *create_info_ptr };
        for i in 0..usize::try_from(create_info.enabled_extension_count)? {
            let ptr = unsafe { *create_info.enabled_extension_names.add(i) };
            let ext_name = unsafe { CStr::from_ptr(ptr) };
            if ext_name.to_bytes() == timespec.as_bytes() {
                has_timespec = true;
            }
            #[cfg(windows)]
            if ext_name.to_bytes() == perf_counter.as_bytes() {
                has_perf_counter = true;
            }
            LogTrace::string(format!("create_instance: extension: {:?}", ext_name));
            extensions.push(ptr);
        }
        let exts = layer.entry.enumerate_extensions()?;
        if exts.khr_convert_timespec_time && !has_timespec {
            extensions.push(timespec.as_ptr());
            LogTrace::string(format!("added extension: {:?}", timespec));
        }
        #[cfg(windows)]
        if exts.khr_win32_convert_performance_counter_time && !has_perf_counter {
            extensions.push(perf_counter.as_ptr());
            LogTrace::string(format!("added extension: {:?}", perf_counter));
        }
        create_info.enabled_extension_names = extensions.as_ptr();
        create_info.enabled_extension_count = extensions.len() as u32;

        let result = unsafe { (layer.entry.fp().create_instance)(&create_info, instance_ptr) };
        if result == sys::Result::SUCCESS {
            let instance_handle = unsafe { *instance_ptr };
            let exts = InstanceExtensions {
                khr_convert_timespec_time: unsafe {
                    raw::ConvertTimespecTimeKHR::load(&layer.entry, instance_handle)
                }
                .ok(),
                #[cfg(windows)]
                khr_win32_convert_performance_counter_time: unsafe {
                    raw::Win32ConvertPerformanceCounterTimeKHR::load(&layer.entry, instance_handle)
                }
                .ok(),
                ..InstanceExtensions::default()
            };
            let instance_result = unsafe {
                safe_openxr::Instance::from_raw(layer.entry.clone(), instance_handle, exts)
            };
            if let Ok(instance) = instance_result {
                layer
                    .instances
                    .insert(instance_handle, Instance::new(instance));
                LogTrace::str("Instance successfully created");
            } else {
                LogWarn::str("Failed to acquire Instance");
            }
        }
        result.into_result()
    })
}

extern "system" fn destroy_instance(instance_handle: sys::Instance) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("destroy_instance").entered();
        let _ = layer
            .instances
            .remove(&instance_handle)
            // Note: ERROR_INSTANCE_LOST is incorrect, because it implies that
            // destroy instance needs to be called later, which will not succeed
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
        // This is already done by above's drop:
        // let result = unsafe { (instance.fp().destroy_instance)(instance_handle) };
        // result.into_result()
        Ok(())
    })
}

fn read_instance(
    layer: &Layer,
    instance_handle: sys::Instance,
) -> Result<&Instance, xr_layer::sys::Result> {
    layer
        .instances
        .get(&instance_handle)
        // Note: ERROR_INSTANCE_LOST is incorrect, because it implies that
        // destroy instance needs to be called later, which will not succeed
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)
}

fn read_instance_mut(
    instances: &mut HashMap<sys::Instance, Instance>,
    instance_handle: sys::Instance,
) -> Result<&mut Instance, xr_layer::sys::Result> {
    instances
        .get_mut(&instance_handle)
        // Note: ERROR_INSTANCE_LOST is incorrect, because it implies that
        // destroy instance needs to be called later, which will not succeed
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)
}

fn subresource_read_instance<T: std::hash::Hash + std::cmp::Eq>(
    layer: &Layer,
    reader: fn(layer: &InstanceRefs) -> &HashMap<T, sys::Instance>,
    handle: T,
) -> Result<&Instance, xr_layer::sys::Result> {
    let instance_handle = reader(&layer.instance_refs)
        .get(&handle)
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
    read_instance(layer, *instance_handle)
}

fn subresource_read_instance_mut<T: std::hash::Hash + std::cmp::Eq>(
    layer: &mut Layer,
    reader: fn(layer: &mut InstanceRefs) -> &mut HashMap<T, sys::Instance>,
    handle: T,
) -> Result<&mut Instance, xr_layer::sys::Result> {
    let instances = &mut layer.instances;
    let instance_handle = reader(&mut layer.instance_refs)
        .get(&handle)
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
    read_instance_mut(instances, *instance_handle)
}

fn instance_ref_delete<T: std::hash::Hash + std::cmp::Eq>(
    layer: &mut Layer,
    reader: fn(layer: &mut InstanceRefs) -> &mut HashMap<T, sys::Instance>,
    handle: T,
) -> Result<&mut Instance, xr_layer::sys::Result> {
    // TODO: deleting instance should also delete sub-resources, and eg. deleting
    // ActionSet should also delete Action, etc.

    let instance_handle = reader(&mut layer.instance_refs)
        .remove(&handle)
        .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
    read_instance_mut(&mut layer.instances, instance_handle)
}

extern "system" fn poll_event(
    instance_handle: sys::Instance,
    event_data: *mut sys::EventDataBuffer,
) -> sys::Result {
    wrap(|layer| {
        let span = trace_span!("poll_event", result = tracing::field::Empty).entered();
        let instance = read_instance(layer, instance_handle)?;

        let result = unsafe { (instance.fp().poll_event)(instance_handle, event_data) };
        span.record_debug("result", result);
        result.into_result()?;
        let result = result.into_result();
        let span = trace_span!("event", event = tracing::field::Empty).entered();
        span.record_debug(
            "event",
            unsafe { XrStructChain::from_ptr(event_data) }.as_debug(&instance.instance),
        );

        let Ok(buf) = unsafe { XrStructChain::from_ptr(event_data) }
            .read_event_data_interaction_profile_changed() else { return result; };
        let Some(session) = instance.sessions.get(&buf.session()) else { return result; };
        let Ok(mut profiles) = session.active_interaction_profiles.write() else {return result;};
        // TODO: remove ? and instead just log and return result
        // TODO: stop hardcoding just left/right
        let left = instance.instance.string_to_path("/user/hand/left")?;
        let right = instance.instance.string_to_path("/user/hand/right")?;
        profiles.clear();
        profiles.insert(left, session.session.current_interaction_profile(left)?);
        profiles.insert(right, session.session.current_interaction_profile(right)?);
        LogTrace::str("Successfully updated active_interaction_profiles");
        result
    })
}

extern "system" fn create_action_set(
    instance_handle: sys::Instance,
    info: *const sys::ActionSetCreateInfo,
    action_set_ptr: *mut sys::ActionSet,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("create_action_set").entered();
        let instance = read_instance_mut(&mut layer.instances, instance_handle)?;

        let result =
            unsafe { (instance.fp().create_action_set)(instance_handle, info, action_set_ptr) };
        if result.into_result().is_ok() {
            let action_set = unsafe { *action_set_ptr };

            // Insert object with related data
            let mut map = instance.action_sets.write()?;
            map.insert(action_set, ActionSet::default());

            // Insert referencing object
            layer
                .instance_refs
                .action_sets
                .insert(action_set, instance_handle);
        }
        result.into_result()
    })
}

extern "system" fn destroy_action_set(action_set: sys::ActionSet) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("create_action_set").entered();

        let instance = subresource_read_instance_mut(layer, |l| &mut l.action_sets, action_set)?;
        instance.action_sets.write()?.remove(&action_set);

        let result = unsafe { (instance.fp().destroy_action_set)(action_set) };
        layer.instance_refs.action_sets.remove(&action_set);

        result.into_result()
    })
}

extern "system" fn create_action(
    action_set_handle: sys::ActionSet,
    info_in: *const sys::ActionCreateInfo,
    out: *mut sys::Action,
) -> sys::Result {
    wrap(|layer| {
        let span = trace_span!("create_action", info = tracing::field::Empty).entered();
        let instance = subresource_read_instance(layer, |l| &l.action_sets, action_set_handle)?;
        span.record_debug(
            "info",
            unsafe { XrStructChain::from_ptr(info_in) }.as_debug(&instance.instance),
        );

        let info = unsafe { XrStructChain::from_ptr(info_in) }.read_action_create_info()?;
        let result =
            unsafe { (instance.fp().create_action)(action_set_handle, info_in, out) }.into_result();
        if result.is_ok() {
            let handle = unsafe { *out };

            let mut map = instance.action_sets.write()?;
            let set = map
                .get_mut(&action_set_handle)
                .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
            set.actions.push(Action {
                handle,

                path: HashMap::default(),
                name: info.action_name()?.to_string(),
                typ: info.action_type().into(),
                localized_name: info.localized_action_name()?.to_string(),
                subaction_paths: info.subaction_paths().collect(),
            });
        }
        result
    })
}

extern "system" fn string_to_path(
    instance_handle: sys::Instance,
    path_string_raw: *const c_char,
    path: *mut sys::Path,
) -> sys::Result {
    xr_wrap(|| {
        let span =
            trace_span!("string_to_path", string = field::Empty, path = field::Empty).entered();
        let str = unsafe { CStr::from_ptr(path_string_raw) }
            .to_str()
            .map_err(|_| sys::Result::ERROR_PATH_INVALID)?;
        span.record("string", format!("{:?}", str));

        let r = LAYER.read()?;
        let layer = (*r).as_ref().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;

        let instance = read_instance(layer, instance_handle)?;

        let result =
            unsafe { (instance.fp().string_to_path)(instance_handle, path_string_raw, path) };
        span.record("path", format!("{:?}", unsafe { *path }));
        result.into_result()
    })
}

extern "system" fn suggest_interaction_profile_bindings(
    instance_handle: sys::Instance,
    suggested_bindings: *const sys::InteractionProfileSuggestedBinding,
) -> sys::Result {
    wrap(|layer| {
        let span = trace_span!(
            "suggest_interaction_profile_bindings",
            suggested_bindings = tracing::field::Empty
        )
        .entered();
        let instance = read_instance(layer, instance_handle)?;
        span.record_debug(
            "suggested_bindings",
            unsafe { XrStructChain::from_ptr(suggested_bindings) }.as_debug(&instance.instance),
        );

        let result = unsafe {
            (instance.fp().suggest_interaction_profile_bindings)(
                instance_handle,
                suggested_bindings,
            )
        };
        result.into_result()?;
        let result = result.into_result();
        let Ok(bindings) = unsafe { XrStructChain::from_ptr(suggested_bindings) }
            .read_interaction_profile_suggested_binding() else {return result;};

        let mut sets = instance.action_sets.write()?;
        // This is inefficient, but it's only done in application startup.
        for binding in bindings.suggested_bindings() {
            for set in sets.values_mut() {
                for action in &mut set.actions {
                    if action.handle == binding.action {
                        action
                            .path
                            .insert(bindings.interaction_profile(), binding.binding);
                    }
                }
            }
        }
        result
    })
}

extern "system" fn create_session(
    instance_handle: sys::Instance,
    create_info_ptr: *const sys::SessionCreateInfo,
    session_ptr: *mut sys::Session,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("create_session").entered();
        let instance = read_instance_mut(&mut layer.instances, instance_handle)?;

        let result = unsafe {
            (instance.fp().create_session)(instance_handle, create_info_ptr, session_ptr)
        };
        if result.into_result().is_ok() {
            let session_handle = unsafe { *session_ptr };
            layer
                .instance_refs
                .sessions
                .insert(session_handle, instance_handle);
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

            LogTrace::str("Creating session");
            instance
                .sessions
                .insert(session_handle, Session::new(session, &layer.trace)?);
            LogTrace::str("Initialized session");
        }
        result.into_result()
    })
}

extern "system" fn destroy_session(session_handle: sys::Session) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("destroy_session").entered();
        let instance = instance_ref_delete(layer, |l| &mut l.sessions, session_handle)?;

        instance
            .sessions
            .remove(&session_handle)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
        Ok(())
        // already handled:
        // ```
        // let result = unsafe { (instance.fp().destroy_session)(session_handle) };
        // result.into_result()
        // ```
    })
}

extern "system" fn begin_session(
    session_handle: sys::Session,
    begin_info_ptr: *const sys::SessionBeginInfo,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("begin_session").entered();
        let instance = subresource_read_instance_mut(layer, |l| &mut l.sessions, session_handle)?;

        let begin_session = instance.fp().begin_session;

        let begin_info = unsafe { *begin_info_ptr };
        let session = instance
            .sessions
            .get_mut(&session_handle)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;

        let result = unsafe { (begin_session)(session_handle, begin_info_ptr) }.into_result();
        if result.is_ok() {
            session.view_configuration_type = begin_info.primary_view_configuration_type;
        }
        result
    })
}

extern "system" fn get_current_interaction_profile(
    session_handle: sys::Session,
    top_level_user_path: sys::Path,
    interaction_profile_ptr: *mut sys::InteractionProfileState,
) -> sys::Result {
    wrap(|layer| {
        let span = trace_span!(
            "get_current_interaction_profile",
            session = ?session_handle,
            top_level_user_path = tracing::field::Empty,
            interaction_profile = tracing::field::Empty
        )
        .entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;
        span.record_debug(
            "top_level_user_path",
            top_level_user_path.as_debug(&instance.instance),
        );

        let result = unsafe {
            (instance.fp().get_current_interaction_profile)(
                session_handle,
                top_level_user_path,
                interaction_profile_ptr,
            )
        }
        .into_result();
        if result.is_ok() {
            span.record_debug(
                "interaction_profile",
                unsafe { XrStructChain::from_ptr(interaction_profile_ptr) }
                    .as_debug(&instance.instance),
            );
        }
        result
    })
}

extern "system" fn sync_actions(
    session_handle: sys::Session,
    sync_info: *const sys::ActionsSyncInfo,
) -> sys::Result {
    wrap(|layer| {
        let span = trace_span!(
            "sync_actions",
            info = tracing::field::Empty,
            result = tracing::field::Empty
        )
        .entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;
        span.record_debug(
            "info",
            unsafe { XrStructChain::from_ptr(sync_info) }.as_debug(&instance.instance),
        );

        let result = unsafe { (instance.fp().sync_actions)(session_handle, sync_info) };
        span.record_debug("result", result.into_result());

        // TODO: trigger upload to server

        result.into_result()
    })
}

extern "system" fn get_action_state_boolean(
    session_handle: sys::Session,
    get_info_ptr: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateBoolean,
) -> sys::Result {
    wrap(|layer| {
        let _span = trace_span!("get_action_state_boolean").entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;

        let get_info = unsafe { XrStructChain::from_ptr(get_info_ptr) };

        info!("{:?}", get_info.as_debug(&instance.instance));

        let result = unsafe {
            (instance.fp().get_action_state_boolean)(session_handle, get_info_ptr, state)
        };
        info!(
            "{:?}",
            unsafe { XrStructChain::from_ptr(state) }.as_debug(&instance.instance)
        );
        result.into_result()
    })
}

extern "system" fn get_action_state_vector2f(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateVector2f,
) -> sys::Result {
    wrap(|layer| {
        let _span = trace_span!("get_action_state_vector2f").entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;

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
    wrap(|layer| {
        let _span = trace_span!("get_action_state_pose").entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;

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
    wrap(|layer| {
        let _span = trace_span!("apply_haptic_feedback").entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;

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

pub(crate) fn with_layer<T, R>(handle: sys::Instance, cb: T) -> anyhow::Result<R>
where
    T: FnOnce(&Instance) -> anyhow::Result<R>,
{
    let r = LAYER
        .read()
        .map_err(|err| anyhow!("Failed to acquire read lock for layer: {:?}", err))?;
    let layer = (*r).as_ref().ok_or(sys::Result::ERROR_RUNTIME_FAILURE)?;
    let instance = read_instance(layer, handle)?;
    layer.trace.wrap(|| cb(instance))
}

extern "system" fn wait_frame(
    session_handle: sys::Session,
    frame_wait_info: *const sys::FrameWaitInfo,
    frame_state_ptr: *mut sys::FrameState,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("wait_frame").entered();
        let instance = subresource_read_instance_mut(layer, |l| &mut l.sessions, session_handle)?;

        let result =
            unsafe { (instance.fp().wait_frame)(session_handle, frame_wait_info, frame_state_ptr) }
                .into_result();

        if result.is_ok() {
            let frame_state = unsafe { *frame_state_ptr };
            if let Some(session) = instance.sessions.get_mut(&session_handle) {
                session.predicted_display_time = frame_state.predicted_display_time;
            }
        }

        result
    })
}

macro_rules! simple_s {
    ($id: ident ($farg: ident: $fty: ty, $($arg: ident: $ty: ty, ) *)) => {
        extern "system" fn $id(
            $farg: $fty,
            $($arg: $ty, )*
        ) -> sys::Result {
            wrap(|layer| {
                let _span = trace_span!(stringify!($id)).entered();
                let instance = subresource_read_instance(layer, |l| &l.sessions, $farg)?;
                let result = unsafe { (instance.fp().$id)($farg, $($arg,)*) }
                .into_result();
                result
            })
        }
    };
}

macro_rules! simple_i {
    ($id: ident ($farg: ident: $fty: ty, $($arg: ident: $ty: ty), *,)) => {
        #[allow(dead_code)]
        extern "system" fn $id(
            $farg: $fty,
            $($arg: $ty, )*
        ) -> sys::Result {
            wrap(|layer| {
                let _span = trace_span!(stringify!($id)).entered();
                let instance = read_instance(layer, $farg)?;
                let result = unsafe { (instance.fp().$id)($farg, $($arg,)*) }
                .into_result();
                result
            })
        }
    };
}

extern "system" fn locate_views(
    session_handle: sys::Session,
    view_locate_info: *const sys::ViewLocateInfo,
    view_state: *mut sys::ViewState,
    view_capacity_input: u32,
    view_count_output: *mut u32,
    view: *mut sys::View,
) -> sys::Result {
    wrap(|layer| {
        let _span = trace_span!("locate_views").entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;
        let session = instance
            .sessions
            .get(&session_handle)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
        let mut info = unsafe { *view_locate_info };
        rewrite_space(session, &mut info.space);
        let result = unsafe {
            (instance.fp().locate_views)(
                session_handle,
                &info,
                view_state,
                view_capacity_input,
                view_count_output,
                view,
            )
        }
        .into_result();
        result
    })
}

fn rewrite_space(session: &Session, space: &mut sys::Space) {
    if let Ok(spaces) = session.application_stage_spaces.read() {
        if spaces.contains(space) {
            match session.space_server.read() {
                Ok(space_server) => {
                    LogTrace::string(format!("Rewriting space: {:?}", space));
                    *space = space_server.as_raw();
                }
                Err(err) => {
                    LogError::string(format!("Couldn't rewrite space: {:?}", err));
                }
            }
        }
    }
}

simple_s!(get_action_state_float(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateFloat,
));

extern "system" fn attach_session_action_sets(
    session_handle: sys::Session,
    attach_info: *const sys::SessionActionSetsAttachInfo,
) -> sys::Result {
    wrap(|layer| {
        let _span = trace_span!("attach_session_action_sets").entered();
        let instance = subresource_read_instance(layer, |l| &l.sessions, session_handle)?;
        let result =
            unsafe { (instance.fp().attach_session_action_sets)(session_handle, attach_info) }
                .into_result();
        if result.is_ok() {
            let sets = instance.action_sets.read()?;
            let session = instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Missing session in instance"))?;

            let info = unsafe { XrStructChain::from_ptr(attach_info) }
                .read_session_action_sets_attach_info()?;

            let mut user_paths = HashMap::new();
            let mut profile_map = HashMap::<sys::Path, InteractionProfile>::default();
            for set_handle in info.action_sets() {
                let Some(set) = sets.get(&set_handle) else { continue; };

                for action in set.clone().actions {
                    for (profile_path, binding_path) in action.path {
                        let profile = profile_map.entry(profile_path).or_insert_with(|| {
                            InteractionProfile {
                                bindings: Vec::new(),
                                path: instance
                                    .instance
                                    .path_to_string(profile_path)
                                    // TODO: is this a problem?
                                    .unwrap_or_else(|err| format!("error: {:?}", err)),
                                path_handle: profile_path,
                            }
                        });

                        let spaces = if let net::ActionType::Pose = action.typ {
                            let mut map = HashMap::new();
                            for subaction_path in action.subaction_paths.iter() {
                                if !user_paths.contains_key(subaction_path) {
                                    user_paths.insert(
                                        *subaction_path,
                                        instance.instance.path_to_string(*subaction_path)?,
                                    );
                                }
                                map.insert(
                                    *subaction_path,
                                    util_create_action_space(
                                        instance,
                                        session_handle,
                                        action.handle,
                                        subaction_path,
                                    )?,
                                );
                            }
                            Some(map)
                        } else {
                            None
                        };

                        profile.bindings.push(local_configuration::Action {
                            ty: action.typ.clone(),
                            name: action.name.clone(),
                            localized_name: action.localized_name.clone(),
                            binding: instance.instance.path_to_string(binding_path)?,
                            spaces,
                        });
                    }
                }
            }

            let conf = LocalConfigurationSnapshot {
                // TODO: this is not atomic and might be wrong.
                // It's okay in Unity's case because event system has a dedicated thread there.
                version: session.local_configuration.borrow().version + 1,
                interaction_profiles: profile_map.values().cloned().collect(),
                user_paths: user_paths.into_iter().collect(),
            };
            session.local_configuration.send_replace(conf);

            // TODO: update configuration on server
        }
        result
    })
}

fn util_create_action_space(
    instance: &Instance,
    session_handle: sys::Session,
    action: sys::Action,
    subaction_path: &sys::Path,
) -> Result<sys::Space, XrWrapError> {
    let mut space = sys::Space::NULL;
    let input = sys::ActionSpaceCreateInfo {
        ty: sys::StructureType::ACTION_SPACE_CREATE_INFO,
        next: ptr::null(),
        action,
        subaction_path: *subaction_path,
        pose_in_action_space: sys::Posef::IDENTITY,
    };
    unsafe { (instance.instance.fp().create_action_space)(session_handle, &input, &mut space) }
        .into_result()?;
    Ok(space)
}

simple_i!(result_to_string(
    instance: sys::Instance,
    value: sys::Result,
    buffer: *mut c_char,
));
simple_i!(structure_type_to_string(
    instance: sys::Instance,
    value: sys::StructureType,
    buffer: *mut c_char,
));
simple_i!(get_instance_properties(
    instance: sys::Instance,
    instance_properties: *mut sys::InstanceProperties,
));
simple_i!(get_system(
    instance: sys::Instance,
    get_info: *const sys::SystemGetInfo,
    system_id: *mut sys::SystemId,
));
simple_i!(get_system_properties(
    instance: sys::Instance,
    system_id: sys::SystemId,
    properties: *mut sys::SystemProperties,
));
simple_s!(end_session(session: sys::Session,));
simple_s!(request_exit_session(session: sys::Session,));
simple_s!(enumerate_reference_spaces(
    session: sys::Session,
    space_capacity_input: u32,
    space_count_output: *mut u32,
    spaces: *mut sys::ReferenceSpaceType,
));
extern "system" fn create_reference_space(
    session: sys::Session,
    create_info: *const sys::ReferenceSpaceCreateInfo,
    space: *mut sys::Space,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("create_reference_space").entered();
        let instance_handle = layer
            .instance_refs
            .sessions
            .get(&session)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?
            .to_owned();
        let instance = read_instance(layer, instance_handle)?;
        let result = unsafe { (instance.fp().create_reference_space)(session, create_info, space) }
            .into_result();
        if result.is_ok() {
            let out_space = unsafe { *space };
            if unsafe { *create_info }.reference_space_type == ReferenceSpaceType::STAGE {
                let session = instance
                    .sessions
                    .get(&session)
                    .ok_or(anyhow!("Missing session in instance"))?;
                // TODO: honor create_info.pose_in_reference_space
                if let Ok(mut spaces) = session.application_stage_spaces.write() {
                    spaces.insert(out_space);
                }
            }
            // Insert referencing object
            layer
                .instance_refs
                .spaces
                .insert(out_space, instance_handle);
            layer
                .instance_refs
                .space_sessions
                .insert(out_space, session);
        }
        result
    })
}
extern "system" fn create_action_space(
    session: sys::Session,
    create_info: *const sys::ActionSpaceCreateInfo,
    space: *mut sys::Space,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!(stringify!(create_action_space)).entered();
        let instance_handle = layer
            .instance_refs
            .sessions
            .get(&session)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?
            .to_owned();
        let instance = read_instance(layer, instance_handle)?;
        let result = unsafe { (instance.fp().create_action_space)(session, create_info, space) }
            .into_result();
        if result.is_ok() {
            let out_space = unsafe { *space };

            // Insert referencing object
            layer
                .instance_refs
                .spaces
                .insert(out_space, instance_handle);
            layer
                .instance_refs
                .space_sessions
                .insert(out_space, session);
        }
        result
    })
}
simple_i!(enumerate_view_configurations(
    instance: sys::Instance,
    system_id: sys::SystemId,
    view_configuration_type_capacity_input: u32,
    view_configuration_type_count_output: *mut u32,
    view_configuration_types: *mut sys::ViewConfigurationType,
));
simple_i!(enumerate_environment_blend_modes(
    instance: sys::Instance,
    system_id: sys::SystemId,
    view_configuration_type: sys::ViewConfigurationType,
    environment_blend_mode_capacity_input: u32,
    environment_blend_mode_count_output: *mut u32,
    environment_blend_modes: *mut sys::EnvironmentBlendMode,
));
simple_i!(get_view_configuration_properties(
    instance: sys::Instance,
    system_id: sys::SystemId,
    view_configuration_type: sys::ViewConfigurationType,
    configuration_properties: *mut sys::ViewConfigurationProperties,
));
simple_i!(enumerate_view_configuration_views(
    instance: sys::Instance,
    system_id: sys::SystemId,
    view_configuration_type: sys::ViewConfigurationType,
    view_capacity_input: u32,
    view_count_output: *mut u32,
    views: *mut sys::ViewConfigurationView,
));
simple_s!(begin_frame(
    session: sys::Session,
    frame_begin_info: *const sys::FrameBeginInfo,
));
simple_s!(end_frame(
    session: sys::Session,
    frame_end_info: *const sys::FrameEndInfo,
));
simple_s!(stop_haptic_feedback(
    session: sys::Session,
    haptic_action_info: *const sys::HapticActionInfo,
));
simple_i!(path_to_string(
    instance: sys::Instance,
    path: sys::Path,
    buffer_capacity_input: u32,
    buffer_count_output: *mut u32,
    buffer: *mut c_char,
));
simple_s!(get_reference_space_bounds_rect(
    session: sys::Session,
    reference_space_type: sys::ReferenceSpaceType,
    bounds: *mut sys::Extent2Df,
));
simple_s!(get_input_source_localized_name(
    session: sys::Session,
    get_info: *const sys::InputSourceLocalizedNameGetInfo,
    buffer_capacity_input: u32,
    buffer_count_output: *mut u32,
    buffer: *mut c_char,
));
simple_s!(enumerate_bound_sources_for_action(
    session_handle: sys::Session,
    enumerate_info: *const sys::BoundSourcesForActionEnumerateInfo,
    source_capacity_input: u32,
    source_count_output: *mut u32,
    sources: *mut sys::Path,
));

extern "system" fn locate_space(
    space: sys::Space,
    base_space: sys::Space,
    time: sys::Time,
    location: *mut sys::SpaceLocation,
) -> sys::Result {
    wrap(|layer| {
        let mut space = space;
        let _span = trace_span!(stringify!(locate_space)).entered();
        let instance_handle = layer
            .instance_refs
            .spaces
            .get(&space)
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
        let instance = read_instance(layer, *instance_handle)?;

        if let Some(session_handle) = layer.instance_refs.space_sessions.get(&space) {
            if let Some(session) = instance.sessions.get(session_handle) {
                rewrite_space(session, &mut space);
            }
        }
        let result = unsafe { (instance.fp().locate_space)(space, base_space, time, location) }
            .into_result();
        result
    })
}
