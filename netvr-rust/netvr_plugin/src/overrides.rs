use crate::{
    implementation::post_poll_event,
    instance::{Instance, Session, ViewData},
    xr_wrap::{xr_wrap, RecordDebug, ResultConvertible, Trace, XrWrapError},
};
use std::{collections::HashMap, ffi::CStr, os::raw::c_char, sync::RwLock};
use tracing::{field, info, trace_span};
use xr_layer::{
    log::{LogError, LogTrace, LogWarn},
    safe_openxr::{self, InstanceExtensions},
    sys, Entry, FnPtr, UnsafeFrom, XrDebug, XrStructChain,
};

struct InstanceRefs {
    pub(crate) sessions: HashMap<sys::Session, sys::Instance>,
    pub(crate) action_sets: HashMap<sys::ActionSet, sys::Instance>,
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
            //.add_override(FnPtr::DestroyActionSet(destroy_action_set))
            //.add_override(FnPtr::EnumerateApiLayerProperties(enumerate_api_layer_properties))
            //.add_override(FnPtr::EnumerateInstanceExtensionProperties(enumerate_instance_extension_properties))
            //.add_override(FnPtr::LocateSpace(locate_space))
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
            instance_refs: InstanceRefs {
                sessions: HashMap::default(),
                action_sets: HashMap::default(),
            },
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
    create_info: *const sys::InstanceCreateInfo,
    instance_ptr: *mut sys::Instance,
) -> sys::Result {
    wrap_mut(|layer| {
        let _span = trace_span!("create_instance").entered();
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
    wrap_mut(|layer| {
        let _span = trace_span!("destroy_instance").entered();
        let _ = layer
            .instances
            .remove(&instance_handle)
            // Note: ERROR_INSTANCE_LOST is incorrect, because it implies that
            // destroy instance needs to be called later, which will not succeed
            .ok_or(sys::Result::ERROR_HANDLE_INVALID)?;
        // This is already done by above's drop:
        //let result = unsafe { (instance.fp().destroy_instance)(instance_handle) };
        //result.into_result()
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
        let _span = trace_span!("poll_event").entered();
        let instance = read_instance(layer, instance_handle)?;

        let result = unsafe { (instance.fp().poll_event)(instance_handle, event_data) };
        if result == sys::Result::SUCCESS {
            let span = trace_span!("event", event = tracing::field::Empty).entered();
            span.record_debug(
                "event",
                unsafe { XrStructChain::from_ptr(event_data) }.as_debug(&instance.instance),
            );
        } else if result == sys::Result::EVENT_UNAVAILABLE {
            let option = post_poll_event(instance).map_err(|err| {
                LogError::string(format!("post_poll_event failed with error {:?}", err));
                sys::Result::EVENT_UNAVAILABLE
            })?;
            if let Some(data) = option {
                unsafe { std::ptr::write(event_data, *data.as_raw()) };
                return sys::Result::SUCCESS.into_result();
            }
        }
        result.into_result()
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
            layer
                .instance_refs
                .action_sets
                .insert(action_set, instance_handle);
        }
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

        let result = unsafe { (instance.fp().create_action)(action_set_handle, info_in, out) };
        result.into_result()
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
        result.into_result()
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

            instance
                .sessions
                .insert(session_handle, Session::new(session, &layer.trace)?);
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
        //let result = unsafe { (instance.fp().destroy_session)(session_handle) };
        //result.into_result()
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

        let result = unsafe {
            (instance.fp().locate_views)(
                session_handle,
                view_locate_info,
                view_state,
                view_capacity_input,
                view_count_output,
                view,
            )
        }
        .into_result();
        if result.is_ok() {
            let mut vec = instance.views.lock().map_err(|err| err.to_string())?;
            vec.clear();
            let size: usize =
                std::cmp::min(unsafe { *view_count_output }, view_capacity_input).try_into()?;
            for i in 0..size {
                let view = unsafe { *view.add(i) };
                vec.push(ViewData {
                    fov: view.fov,
                    pose: view.pose,
                });
            }
        }
        result
    })
}

pub(crate) fn with_layer<T, R>(handle: sys::Instance, cb: T) -> Result<R, XrWrapError>
where
    T: FnOnce(&Instance) -> Result<R, XrWrapError>,
{
    let r = LAYER.read()?;
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
                session.time = frame_state.predicted_display_time;
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

simple_s!(get_action_state_float(
    session_handle: sys::Session,
    get_info: *const sys::ActionStateGetInfo,
    state: *mut sys::ActionStateFloat,
));
simple_s!(attach_session_action_sets(
    session_handle: sys::Session,
    attach_info: *const sys::SessionActionSetsAttachInfo,
));
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
simple_s!(create_reference_space(
    session: sys::Session,
    create_info: *const sys::ReferenceSpaceCreateInfo,
    space: *mut sys::Space,
));
simple_s!(create_action_space(
    session: sys::Session,
    create_info: *const sys::ActionSpaceCreateInfo,
    space: *mut sys::Space,
));
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