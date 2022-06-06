use crate::log::LogError;
use crate::log::LogInfo;
use crate::log::LogWarn;
use crate::utils::xr_wrap;
use crate::utils::ResultConvertible;
use crate::xr_functions::decode_xr_result;
use crate::xr_functions::{self, XrFunctions, XrInstanceFunctions};
use crate::xr_structures::*;

use openxr_sys::pfn;
use std::collections::hash_map::HashMap;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::RwLock;

lazy_static! {
    // store get_instance_proc_addr
    static ref FUNCTIONS: RwLock<Option<XrFunctions>> = RwLock::new(Option::None);
}

fn get_functions(caller: &str) -> Result<XrFunctions, openxr_sys::Result> {
    let r = FUNCTIONS.read().unwrap();
    match *r {
        Some(v) => Ok(v),
        None => {
            LogError::string(format!(
                "{} was called before setting up pointer to xrGetInstanceProcAddr",
                caller
            ));
            Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
        }
    }
}

fn parse_input_string<'a>(name_ptr: *const c_char) -> Option<&'a str> {
    match unsafe { CStr::from_ptr(name_ptr) }.to_str() {
        Ok(val) => Some(val),
        Err(error) => {
            LogWarn::string(format!(
                "Failed to parse string input as UTF8. Error: {}",
                error.source().unwrap(),
            ));
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct ImplementationInstancePtr(pub *mut ::std::os::raw::c_void);
unsafe impl Send for ImplementationInstancePtr {}
unsafe impl Sync for ImplementationInstancePtr {}

#[derive(Clone, Copy)]
struct LayerInstance {
    pub implementation: ImplementationInstancePtr,
    pub functions: XrInstanceFunctions,
}

pub struct XrLayerLoader<Implementation> {
    _never_instantiated: Implementation,
}

lazy_static! {
    static ref N_CREATE_IMPLEMENTATION_INSTANCE: RwLock<Option<fn() -> ImplementationInstancePtr>> =
        RwLock::new(None);
    static ref N_INSTANCES: RwLock<HashMap<u64, LayerInstance>> = RwLock::new(HashMap::new());
    static ref N_SESSIONS: RwLock<HashMap<u64, openxr_sys::Instance>> = RwLock::new(HashMap::new());
}

pub trait ImplementationTrait {
    fn new() -> Self;
}

impl<Implementation: ImplementationTrait> XrLayerLoader<Implementation> {
    fn get_instance(
        caller: &str,
        instance_handle: openxr_sys::Instance,
    ) -> Result<LayerInstance, openxr_sys::Result> {
        let r = N_INSTANCES.read().unwrap();
        let handle = instance_handle.into_raw();
        match (*r).get(&handle) {
            Some(v) => Ok(*v),
            None => {
                LogError::string(format!(
                    "{}: Can't find instance with handle {}. Maybe it was destroyed already?",
                    caller, handle,
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }
    }

    fn get_session(
        caller: &str,
        session_handle: openxr_sys::Session,
    ) -> Result<(LayerInstance, openxr_sys::Instance), openxr_sys::Result> {
        let r = N_SESSIONS.read().unwrap();
        let handle = session_handle.into_raw();
        match (*r).get(&handle) {
            Some(v) => {
                let value = *v;
                let instance = Self::get_instance(caller, value)?;
                Ok((instance, value))
            }
            None => {
                LogError::string(format!(
                    "{}: Can't find session with handle {}. Maybe it was destroyed already?",
                    caller, handle,
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }
    }

    fn create_implementation_instance() -> ImplementationInstancePtr {
        let val = Box::<Implementation>::new(Implementation::new());
        unsafe { std::mem::transmute(Box::into_raw(val)) }
    }

    // this gets called from unity to give us option to override basically any openxr function
    pub fn hook_get_instance_proc_addr(
        func_in: Option<openxr_sys::pfn::GetInstanceProcAddr>,
        // this is useful in unity editor case where we want to unload the library
        // on xrInstanceDestroy, but that is before the actual destroy is called.
        // This means that we:
        //  - call netvr_manual_destroy_instance
        //  - unload the dll
        //  - your game engine calls xrInstanceDestroy from runtime
        // If you aren't doing any dll re-loading you should set this to true, which
        // results in following steps:
        //  - your game engine calls xrInstanceDestroy from netvr
        //  - netvr performs appropriate cleanup
        //  - netvr calls runtime's xrInstanceDestroy
        // This all happens automatically.
        automatic_destroy: bool,
    ) -> Option<openxr_sys::pfn::GetInstanceProcAddr> {
        LogInfo::str("isbl_netvr_hook_get_instance_proc_addr");
        let func = match func_in {
            Some(f) => f,
            None => {
                LogError::str("xrGetInstanceProcAddr is null. Expected valid function pointer.");
                return None;
            }
        };
        {
            let mut w = N_CREATE_IMPLEMENTATION_INSTANCE.write().unwrap();
            if let Some(_) = *w {
                LogError::str("hook_get_instance_proc_addr can only be called once");
                return None;
            }
            *w = Some(Self::create_implementation_instance);
        }

        let mut value = match crate::xr_functions::load(func) {
            Ok(v) => v,
            Err(error) => {
                LogError::string(format!(
                    "Failed to initialize. Disabling netvr layer.\n  Original error: {}",
                    error
                ));
                return func_in;
            }
        };
        value.automatic_destroy = automatic_destroy;
        let mut w = FUNCTIONS.write().unwrap();
        *w = Some(value);
        return Some(Self::override_get_instance_proc_addr);
    }

    pub fn netvr_manual_destroy_instance(instance_handle: openxr_sys::Instance) {
        let status = Self::override_destroy_instance(instance_handle);
        if status == openxr_sys::Result::SUCCESS {
            LogInfo::string(format!(
                "Instance {} was successfully destroyed",
                instance_handle.into_raw()
            ));
        } else {
            LogError::string(format!(
                "Instance {} was destroyed with non-zero status {}",
                instance_handle.into_raw(),
                crate::xr_functions::decode_xr_result(status)
            ));
        }
    }

    // here we can return something different to override any openxr function
    extern "system" fn override_get_instance_proc_addr(
        instance_handle: openxr_sys::Instance,
        name_ptr: *const c_char,
        function: *mut Option<openxr_sys::pfn::VoidFunction>,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let fallback_fns = |fns: XrFunctions| {
                unsafe { (fns.get_instance_proc_addr)(instance_handle, name_ptr, function) }
                    .into_result()
            };
            let fallback = || {
                let fns = get_functions("xrGetInstanceProcAddr")?;
                fallback_fns(fns)
            };

            let name = match parse_input_string(name_ptr) {
                Some(val) => val,
                None => {
                    return fallback();
                }
            };

            if !name.starts_with("xr") {
                LogWarn::string(format!(
                    "xrGetInstanceProcAddr can only handle functions starting with xr. Got: {}",
                    name,
                ));
                return fallback();
            }

            macro_rules! check {
                ($t: ty, $func: expr) => {{
                    if stringify!($t)[5..] == name[2..] {
                        LogInfo::string(format!(
                            "xrGetInstanceProcAddr: Returning {} for {}",
                            stringify!($func),
                            name
                        ));
                        let func: $t = $func; // guard that $func is of correct type
                        unsafe { *function = Some(std::mem::transmute(func)) };
                        return Ok(());
                    }
                }};
            }

            if instance_handle == openxr_sys::Instance::NULL {
                check!(pfn::CreateInstance, Self::override_create_instance);
                // "xrEnumerateInstanceExtensionProperties"
                // "xrEnumerateApiLayerProperties"
                return fallback();
            }

            let r = N_INSTANCES.read().unwrap();
            let handle = instance_handle.into_raw();
            if (*r).get(&handle).is_none() {
                // do not return overrides for uninitialized instances
                return fallback();
            };

            if name == "xrDestroyInstance" {
                let fns = get_functions("xrGetInstanceProcAddr")?;
                // do not hook xrDestroyInstance if automatic_destroy is disabled
                if fns.automatic_destroy {
                    check!(pfn::DestroyInstance, Self::override_destroy_instance);
                } else {
                    LogInfo::str("Skipping automatic destroy registration. You'll have to call netvr_manual_destroy_instance manually.");
                    return fallback_fns(fns);
                }
            }
            #[rustfmt::skip]
        let checks = || {
            check!(pfn::PollEvent, Self::override_poll_event);
            check!(pfn::CreateActionSet, Self::override_create_action_set);
            //check!(pfn::CreateAction, override_create_action);
            check!(pfn::StringToPath, Self::override_string_to_path);
            check!(pfn::SuggestInteractionProfileBindings, Self::override_suggest_interaction_profile_bindings);
            check!(pfn::AttachSessionActionSets, Self::override_attach_session_action_sets);
            check!(pfn::SyncActions, Self::override_sync_actions);
            check!(pfn::GetActionStateBoolean, Self::override_get_action_state_boolean);
            check!(pfn::ApplyHapticFeedback, Self::override_apply_haptic_feedback);
            check!(pfn::CreateSession, Self::override_create_session);
            check!(pfn::DestroySession, Self::override_destroy_session);
            fallback()
        };
            checks()
        })
    }

    extern "system" fn override_create_instance(
        create_info: *const openxr_sys::InstanceCreateInfo,
        instance_ptr: *mut openxr_sys::Instance,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let fns = get_functions("xrCreateInstance")?;

            let result = unsafe { (fns.create_instance)(create_info, instance_ptr) };
            let instance: openxr_sys::Instance = unsafe { *instance_ptr };
            if result != openxr_sys::Result::SUCCESS {
                LogError::string(format!(
                "Underlying xrCreateInstance returned non-success error code {}. Netvr won't be enabled for instance {}.",
                xr_functions::decode_xr_result(result),
                instance.into_raw(),
            ));
                return result.into_result();
            };
            let value =
                match super::xr_functions::load_instance(instance, fns.get_instance_proc_addr) {
                    Ok(v) => v,
                    Err(error) => {
                        LogError::string(format!(
                    "load_instance failed with error {}. Netvr won't be enabled for instance {}.",
                    error,
                    instance.into_raw(),
                ));
                        return result.into_result();
                    }
                };
            let create_implementation_instance = {
                let r = N_CREATE_IMPLEMENTATION_INSTANCE.read().unwrap();
                match *r {
                    Some(v) => Ok(v),
                    None => {
                        LogError::str("Failed to read create_instance");
                        Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
                    }
                }
            }?;
            let mut w = N_INSTANCES.write().unwrap();
            (*w).insert(
                instance.into_raw(),
                LayerInstance {
                    implementation: create_implementation_instance(),
                    functions: value,
                },
            );
            result.into_result()
        })
    }

    extern "system" fn override_destroy_instance(
        instance_handle: openxr_sys::Instance,
    ) -> openxr_sys::Result {
        let mut w = N_INSTANCES.write().unwrap();
        let handle = instance_handle.into_raw();
        let instance = match (*w).get(&handle) {
            Some(v) => *v,
            None => {
                LogError::string(format!(
                    "Can't find instance with handle {}. Maybe it was destroyed already?",
                    handle
                ));
                return openxr_sys::Result::ERROR_HANDLE_INVALID;
            }
        };
        let result = unsafe { (instance.functions.destroy_instance)(instance_handle) };
        (*w).remove(&handle);
        return result;
    }

    extern "system" fn override_poll_event(
        instance_handle: openxr_sys::Instance,
        event_data: *mut openxr_sys::EventDataBuffer,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let instance = Self::get_instance("xrPollEvent", instance_handle)?;
            let result = unsafe { (instance.functions.poll_event)(instance_handle, event_data) }
                .into_result();
            if result.is_ok() {
                for ptr in XrIterator::event_data_buffer(event_data) {
                    let ptr: DecodedStruct = ptr;
                    if let Some(d) = ptr.into_event_data_session_state_changed() {
                        LogInfo::string(format!("Event(SessionStateChanged): {:#?}", d.state));
                    } else if let Some(d) = ptr.into_event_data_interaction_profile_changed() {
                        LogInfo::string(format!(
                            "Event(InteractionProfileChanged): {:#?}",
                            d.session
                        ));
                    } else {
                        LogInfo::string(format!("Event({:#?})", ptr.ty));
                    }
                }
            }
            result
        })
    }

    extern "system" fn override_create_action_set(
        instance_handle: openxr_sys::Instance,
        info: *const openxr_sys::ActionSetCreateInfo,
        out: *mut openxr_sys::ActionSet,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let instance = Self::get_instance("xrCreateActionSet", instance_handle)?;
            let result =
                unsafe { (instance.functions.create_action_set)(instance_handle, info, out) };
            LogInfo::string(format!(
                "xrCreateActionSet {:#?} -> {}",
                info,
                xr_functions::decode_xr_result(result)
            ));
            result.into_result()
        })
    }
    /*
    extern "system" fn override_create_action(
        action_set_handle: openxr_sys::ActionSet,
        info: *const openxr_sys::ActionCreateInfo,
        out: *mut openxr_sys::Action,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let instance = get_instance("xrCreateAction", instance_handle)?;
            let result =
                unsafe { (instance.create_action)(action_set_handle, info, out) }.into_result();
            result
        })
    }
    */
    extern "system" fn override_string_to_path(
        instance_handle: openxr_sys::Instance,
        path_string_raw: *const c_char,
        path: *mut openxr_sys::Path,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let instance = Self::get_instance("xrStringToPath", instance_handle)?;
            let result = unsafe {
                (instance.functions.string_to_path)(instance_handle, path_string_raw, path)
            };
            if let Some(path_string) = parse_input_string(path_string_raw) {
                LogInfo::string(format!(
                    "xrStringToPath \"{}\" -> {}",
                    path_string,
                    xr_functions::decode_xr_result(result)
                ));
            }
            result.into_result()
        })
    }

    extern "system" fn override_suggest_interaction_profile_bindings(
        instance_handle: openxr_sys::Instance,
        suggested_bindings: *const openxr_sys::InteractionProfileSuggestedBinding,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let instance =
                Self::get_instance("xrSuggestInteractionProfileBindings", instance_handle)?;
            let result = unsafe {
                (instance.functions.suggest_interaction_profile_bindings)(
                    instance_handle,
                    suggested_bindings,
                )
            };
            LogInfo::string(format!(
                "xrSuggestInteractionProfileBindings {:#?} -> {}",
                suggested_bindings,
                xr_functions::decode_xr_result(result)
            ));
            result.into_result()
        })
    }

    extern "system" fn override_attach_session_action_sets(
        session_handle: openxr_sys::Session,
        attach_info: *const openxr_sys::SessionActionSetsAttachInfo,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let (instance, _) = Self::get_session("xrAttachSessionActionSets", session_handle)?;
            let result = unsafe {
                (instance.functions.attach_session_action_sets)(session_handle, attach_info)
            };
            LogInfo::string(format!(
                "xrAttachSessionActionSets {:#?} -> {}",
                attach_info,
                xr_functions::decode_xr_result(result)
            ));
            result.into_result()
        })
    }

    extern "system" fn override_sync_actions(
        session_handle: openxr_sys::Session,
        sync_info: *const openxr_sys::ActionsSyncInfo,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let (instance, _) = Self::get_session("xrSyncActions", session_handle)?;
            let result = unsafe { (instance.functions.sync_actions)(session_handle, sync_info) };
            LogInfo::string(format!(
                "xrSyncActions {:#?} -> {}",
                sync_info,
                xr_functions::decode_xr_result(result)
            ));
            result.into_result()
        })
    }

    extern "system" fn override_get_action_state_boolean(
        session_handle: openxr_sys::Session,
        get_info: *const openxr_sys::ActionStateGetInfo,
        state: *mut openxr_sys::ActionStateBoolean,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let (instance, _) = Self::get_session("xrGetActionStateBoolean", session_handle)?;
            let result = unsafe {
                (instance.functions.get_action_state_boolean)(session_handle, get_info, state)
            };
            LogInfo::string(format!(
                "xrGetActionStateBoolean {:#?} -> {}",
                get_info,
                xr_functions::decode_xr_result(result)
            ));
            result.into_result()
        })
    }

    extern "system" fn override_apply_haptic_feedback(
        session_handle: openxr_sys::Session,
        haptic_action_info: *const openxr_sys::HapticActionInfo,
        haptic_feedback: *const openxr_sys::HapticBaseHeader,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let (instance, _) = Self::get_session("xrApplyHapticFeedback", session_handle)?;
            let result = unsafe {
                (instance.functions.apply_haptic_feedback)(
                    session_handle,
                    haptic_action_info,
                    haptic_feedback,
                )
            };
            LogInfo::string(format!(
                "xrApplyHapticFeedback {:#?} -> {}",
                haptic_action_info,
                decode_xr_result(result)
            ));
            result.into_result()
        })
    }

    unsafe extern "system" fn override_create_session(
        instance_handle: openxr_sys::Instance,
        create_info: *const openxr_sys::SessionCreateInfo,
        session_ptr: *mut openxr_sys::Session,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let instance = Self::get_instance("xrCreateSession", instance_handle)?;
            let result = unsafe {
                (instance.functions.create_session)(instance_handle, create_info, session_ptr)
            }
            .into_result();
            if let Err(error) = result {
                LogInfo::string(format!(
                    "xrCreateSession {:#?} -> {}",
                    create_info,
                    xr_functions::decode_xr_result(error)
                ));
            } else {
                let mut w = N_SESSIONS.write().unwrap();
                let session: openxr_sys::Session = unsafe { *session_ptr };
                LogInfo::string(format!(
                    "xrCreateSession {:#?} -> {}",
                    create_info,
                    session.into_raw()
                ));
                (*w).insert(session.into_raw(), instance_handle);
            }
            result
        })
    }

    unsafe extern "system" fn override_destroy_session(
        session_handle: openxr_sys::Session,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let (instance, _) = Self::get_session("xrDestroySession", session_handle)?;
            let result = unsafe { (instance.functions.destroy_session)(session_handle) };
            LogInfo::string(format!("xrDestroySession -> {}", decode_xr_result(result)));

            let mut w = N_SESSIONS.write().unwrap();
            let handle = session_handle.into_raw();
            (*w).remove(&handle);
            result.into_result()
        })
    }
}
