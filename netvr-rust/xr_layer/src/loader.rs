use crate::{
    loader_globals::{GlobalMaps, ImplementationInstancePtr, LayerInstance},
    log::{LogError, LogInfo, LogTrace, LogWarn},
    utils::{xr_wrap, ResultConvertible, ResultToWarning},
    xr_functions::{self, decode_xr_result},
    xr_structures::*,
    LayerImplementation, XrResult,
};

use openxr_sys::pfn;
use std::{error::Error, ffi::CStr, os::raw::c_char, sync::RwLock};

struct LoaderRoot {
    pub entry: openxr::Entry,
    // This probably should not be in this struct, but is the easiest place to
    // put it. In case configuration options grow you should consider creating
    // struct which would contain those and XrFunctions.
    pub automatic_destroy: bool,
}

impl LoaderRoot {
    pub fn create(func: pfn::GetInstanceProcAddr) -> Result<Self, String> {
        Ok(Self {
            entry: unsafe { openxr::Entry::from_get_instance_proc_addr(func) }
                .map_err(decode_xr_result)?,
            automatic_destroy: true,
        })
    }
}

lazy_static! {
    // store get_instance_proc_addr
    static ref LOADER_ROOT: RwLock<Option<LoaderRoot>> = RwLock::new(Option::None);
}

struct LoaderRootLock<'a> {
    pub guard: std::sync::RwLockReadGuard<'a, Option<LoaderRoot>>,
    pub caller: &'static str,
}

impl<'a> LoaderRootLock<'a> {
    pub fn read(&'a self) -> XrResult<&'a LoaderRoot> {
        match &*self.guard {
            Some(v) => Ok(v),
            None => {
                LogError::string(format!(
                    "{} was called before setting up pointer to xrGetInstanceProcAddr",
                    self.caller
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }
    }
}

fn get_functions(caller: &'static str) -> XrResult<LoaderRootLock> {
    Ok(LoaderRootLock {
        guard: match LOADER_ROOT.read() {
            Ok(v) => Ok(v),
            Err(err) => {
                LogError::string(format!(
                    "{}: Failed to acquire read lock on global instances array. {:}",
                    caller, err
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }?,
        caller,
    })
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

pub struct XrLayerLoader<Implementation> {
    _never_instantiated: Implementation,
}

type CreateImplementationInstance =
    fn(instance: &openxr::Instance) -> Option<ImplementationInstancePtr>;

lazy_static! {
    static ref N_CREATE_IMPLEMENTATION_INSTANCE: RwLock<Option<CreateImplementationInstance>> =
        RwLock::new(None);
    static ref GLOBALS: GlobalMaps = GlobalMaps::new();
}

impl<Implementation: LayerImplementation> XrLayerLoader<Implementation> {
    fn create_implementation_instance(
        instance: &openxr::Instance,
    ) -> Option<ImplementationInstancePtr> {
        let val = Box::<Implementation>::new(Implementation::new(instance));
        Some(unsafe { std::mem::transmute(Box::into_raw(val)) })
    }

    fn finish_implementation_instance(ptr: Option<ImplementationInstancePtr>) {
        if let Some(value) = ptr {
            let _p = unsafe {
                Box::from_raw(std::mem::transmute::<
                    ImplementationInstancePtr,
                    *mut Implementation,
                >(value))
            };
        }
    }

    fn read_implementation(instance: &LayerInstance) -> XrResult<&Implementation> {
        match instance.implementation.clone() {
            Some(ptr) => Ok(unsafe { &*std::mem::transmute::<_, *mut Implementation>(ptr) }),
            None => Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE),
        }
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
        LogTrace::str("isbl_netvr_hook_get_instance_proc_addr");
        let func = match func_in {
            Some(f) => f,
            None => {
                LogError::str("xrGetInstanceProcAddr is null. Expected valid function pointer.");
                return None;
            }
        };
        {
            let mut w = N_CREATE_IMPLEMENTATION_INSTANCE.write().unwrap();
            if w.is_some() {
                LogError::str("hook_get_instance_proc_addr can only be called once");
                return None;
            }
            *w = Some(Self::create_implementation_instance);
        }

        let mut value = match LoaderRoot::create(func) {
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
        let mut w = LOADER_ROOT.write().unwrap();
        *w = Some(value);
        Some(Self::override_get_instance_proc_addr)
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
            let fallback_fns = |fns: &LoaderRoot| {
                unsafe {
                    (fns.entry.fp().get_instance_proc_addr)(instance_handle, name_ptr, function)
                }
                .into_result()
            };
            let fallback = || {
                let lock = get_functions("xrGetInstanceProcAddr")?;
                let loader_root = lock.read()?;
                fallback_fns(loader_root)
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
                        LogTrace::string(format!(
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

            let lock = GLOBALS.get_instance_direct("xrGetInstanceProcAddr", instance_handle)?;
            if lock.read().is_err() {
                // do not return overrides for uninitialized instances
                return fallback();
            };

            if name == "xrDestroyInstance" {
                let lock = get_functions("xrGetInstanceProcAddr")?;
                let loader_root = lock.read()?;
                // do not hook xrDestroyInstance if automatic_destroy is disabled
                if loader_root.automatic_destroy {
                    check!(pfn::DestroyInstance, Self::override_destroy_instance);
                } else {
                    LogInfo::str("Skipping automatic destroy registration. You'll have to call netvr_manual_destroy_instance manually.");
                    return fallback_fns(loader_root);
                }
            }
            #[rustfmt::skip]
            let checks = || {
                check!(pfn::PollEvent, Self::override_poll_event);
                check!(pfn::CreateActionSet, Self::override_create_action_set);
                check!(pfn::CreateAction, Self::override_create_action);
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
            let lock = get_functions("xrCreateInstance")?;
            let loader_root = lock.read()?;

            let result =
                unsafe { (loader_root.entry.fp().create_instance)(create_info, instance_ptr) };
            let instance_handle: openxr_sys::Instance = unsafe { *instance_ptr };
            if result != openxr_sys::Result::SUCCESS {
                LogError::string(format!(
                "Underlying xrCreateInstance returned non-success error code {}. Netvr won't be enabled for instance {}.",
                xr_functions::decode_xr_result(result),
                instance_handle.into_raw(),
            ));
                return result.into_result();
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
            let instance = unsafe {
                // TODO: use actual extensions
                let extension_set = openxr::ExtensionSet::default();
                let extensions = openxr::InstanceExtensions::load(
                    &loader_root.entry,
                    instance_handle,
                    &extension_set,
                )?;
                openxr::Instance::from_raw(loader_root.entry.clone(), instance_handle, extensions)
            }?;
            if let Err(error) = GLOBALS.insert_instance(
                instance_handle,
                LayerInstance {
                    implementation: create_implementation_instance(&instance),
                    instance,
                },
            ) {
                LogError::string(format!("Failed to insert instance into context. This will mean that netvr is effectively disabled. Cause: {}", error));
            }
            result.into_result()
        })
    }

    extern "system" fn override_destroy_instance(
        instance_handle: openxr_sys::Instance,
    ) -> openxr_sys::Result {
        xr_wrap(|| match GLOBALS.remove_instance(instance_handle) {
            Ok(mut v) => {
                Self::finish_implementation_instance(v.implementation.take());
                unsafe { (v.instance.fp().destroy_instance)(instance_handle) }.into_result()
            }
            Err(err) => {
                LogError::string(format!("destroy instance failed with error: {}", err));
                Err(openxr_sys::Result::ERROR_HANDLE_INVALID)
            }
        })
    }

    extern "system" fn override_poll_event(
        instance_handle: openxr_sys::Instance,
        event_data: *mut openxr_sys::EventDataBuffer,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let lock = GLOBALS.get_instance_direct("xrPollEvent", instance_handle)?;
            let instance = lock.read()?;
            let result =
                unsafe { (instance.instance.fp().poll_event)(instance_handle, event_data) }
                    .into_result();
            if result.is_ok() {
                for ptr in XrIterator::event_data_buffer(event_data) {
                    let ptr: DecodedStruct = ptr;
                    if let Some(d) = ptr.read_event_data_session_state_changed() {
                        LogTrace::string(format!("Event(SessionStateChanged): {:#?}", d.state));
                    } else if let Some(d) = ptr.read_event_data_interaction_profile_changed() {
                        LogTrace::string(format!(
                            "Event(InteractionProfileChanged): {:#?}",
                            d.session
                        ));
                    } else {
                        LogTrace::string(format!("Event({:#?})", ptr.ty));
                    }
                }
            }
            result
        })
    }

    extern "system" fn override_create_action_set(
        instance_handle: openxr_sys::Instance,
        info: *const openxr_sys::ActionSetCreateInfo,
        action_set_ptr: *mut openxr_sys::ActionSet,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let result = {
                let lock = GLOBALS.get_instance_direct("xrCreateActionSet", instance_handle)?;
                let instance = lock.read()?;
                unsafe {
                    (instance.instance.fp().create_action_set)(
                        instance_handle,
                        info,
                        action_set_ptr,
                    )
                }
            };

            LogTrace::string(format!(
                "xrCreateActionSet {:#?} -> {}",
                info,
                xr_functions::decode_xr_result(result)
            ));
            let decoded_result = result.into_result();

            if let Ok(()) = decoded_result {
                let session = unsafe { *action_set_ptr };
                GLOBALS
                    .insert_instance_reference("xrCreateActionSet", session, instance_handle)
                    .warn_on_err("insert_instance_reference");
            }
            decoded_result
        })
    }

    extern "system" fn override_create_action(
        action_set_handle: openxr_sys::ActionSet,
        info: *const openxr_sys::ActionCreateInfo,
        out: *mut openxr_sys::Action,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let lock = GLOBALS.get_instance("xrCreateAction", action_set_handle)?;
            let instance = lock.read()?;
            let implementation = Self::read_implementation(instance)?;

            implementation.create_action(crate::CreateAction {
                instance: instance.instance.clone().into(),
                action_set_handle,
                info,
                out,
            })
        })
    }

    extern "system" fn override_string_to_path(
        instance_handle: openxr_sys::Instance,
        path_string_raw: *const c_char,
        path: *mut openxr_sys::Path,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let lock = GLOBALS.get_instance_direct("xrStringToPath", instance_handle)?;
            let instance = lock.read()?;
            let result = unsafe {
                (instance.instance.fp().string_to_path)(instance_handle, path_string_raw, path)
            };
            //if let Some(path_string) = parse_input_string(path_string_raw) {
            //    LogTrace::string(format!(
            //        "xrStringToPath \"{}\" -> {}",
            //        path_string,
            //        xr_functions::decode_xr_result(result)
            //    ));
            //}
            result.into_result()
        })
    }

    extern "system" fn override_suggest_interaction_profile_bindings(
        instance_handle: openxr_sys::Instance,
        suggested_bindings: *const openxr_sys::InteractionProfileSuggestedBinding,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let lock = GLOBALS
                .get_instance_direct("xrSuggestInteractionProfileBindings", instance_handle)?;
            let instance = lock.read()?;
            let result = unsafe {
                (instance.instance.fp().suggest_interaction_profile_bindings)(
                    instance_handle,
                    suggested_bindings,
                )
            };
            LogTrace::string(format!(
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
            let lock = GLOBALS.get_instance("xrAttachSessionActionSets", session_handle)?;
            let instance = lock.read()?;
            let result = unsafe {
                (instance.instance.fp().attach_session_action_sets)(session_handle, attach_info)
            };
            LogTrace::string(format!(
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
            let lock = GLOBALS.get_instance("xrSyncActions", session_handle)?;
            let instance = lock.read()?;
            let implementation = Self::read_implementation(instance)?;

            implementation.sync_actions(crate::SyncActions {
                instance: instance.instance.clone().into(),
                sync_info,
                session_handle,
            })
        })
    }

    extern "system" fn override_get_action_state_boolean(
        session_handle: openxr_sys::Session,
        get_info: *const openxr_sys::ActionStateGetInfo,
        state: *mut openxr_sys::ActionStateBoolean,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let lock = GLOBALS.get_instance("xrGetActionStateBoolean", session_handle)?;
            let instance = lock.read()?;
            let result = unsafe {
                (instance.instance.fp().get_action_state_boolean)(session_handle, get_info, state)
            };
            LogTrace::string(format!(
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
            let lock = GLOBALS.get_instance("xrApplyHapticFeedback", session_handle)?;
            let instance = lock.read()?;
            let result = unsafe {
                (instance.instance.fp().apply_haptic_feedback)(
                    session_handle,
                    haptic_action_info,
                    haptic_feedback,
                )
            };
            LogTrace::string(format!(
                "xrApplyHapticFeedback {:#?} -> {}",
                haptic_action_info,
                decode_xr_result(result)
            ));
            result.into_result()
        })
    }

    extern "system" fn override_create_session(
        instance_handle: openxr_sys::Instance,
        create_info: *const openxr_sys::SessionCreateInfo,
        session_ptr: *mut openxr_sys::Session,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let result = {
                let lock = GLOBALS.get_instance_direct("xrCreateSession", instance_handle)?;
                let instance = lock.read()?;
                unsafe {
                    (instance.instance.fp().create_session)(
                        instance_handle,
                        create_info,
                        session_ptr,
                    )
                }
            }
            .into_result();

            if let Err(error) = result {
                LogTrace::string(format!(
                    "xrCreateSession {:#?} -> {}",
                    create_info,
                    xr_functions::decode_xr_result(error)
                ));
            } else {
                let session: openxr_sys::Session = unsafe { *session_ptr };
                GLOBALS
                    .insert_instance_reference("xrCreateSession", session, instance_handle)
                    .warn_on_err("insert_instance_reference");
                LogTrace::string(format!(
                    "xrCreateSession {:#?} -> {}",
                    create_info,
                    session.into_raw()
                ));
            }
            result
        })
    }

    extern "system" fn override_destroy_session(
        session_handle: openxr_sys::Session,
    ) -> openxr_sys::Result {
        xr_wrap(|| {
            let lock = GLOBALS.remove_instance_reference("xrDestroySession", session_handle)?;
            let instance = lock.read()?;
            let result = unsafe { (instance.instance.fp().destroy_session)(session_handle) };

            LogTrace::string(format!("xrDestroySession -> {}", decode_xr_result(result)));

            result.into_result()
        })
    }
}
