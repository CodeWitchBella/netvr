use std::clone::Clone;
use std::ffi::CString;
use std::marker::Copy;

use openxr_sys::pfn;

#[derive(Clone, Copy)]
pub struct XrInstanceFunctions {
    pub destroy_instance: pfn::DestroyInstance,
}

#[derive(Clone, Copy)]
pub struct XrFunctions {
    pub get_instance_proc_addr: pfn::GetInstanceProcAddr,
    pub enumerate_instance_extension_properties: pfn::EnumerateInstanceExtensionProperties,
    pub enumerate_api_layer_properties: pfn::EnumerateApiLayerProperties,
    pub create_instance: pfn::CreateInstance,
}

macro_rules! find_and_cast {
    ($func: expr, $name: expr, $t: ty) => {{
        let func: pfn::GetInstanceProcAddr = $func;
        let name: &str = $name;
        let raw = call_get_instance_proc_addr(openxr_sys::Instance::NULL, func, name);
        match raw {
            Ok(f) => unsafe { std::mem::transmute::<pfn::VoidFunction, $t>(f) },
            Err(error) => {
                return Err(format!(
                    "Failed to load {} with error {}",
                    name,
                    decode_xr_result(error)
                ));
            }
        }
    }};
    ($instance: expr, $func: expr, $name: expr, $t: ty) => {{
        let instance: openxr_sys::Instance = $instance;
        let func: pfn::GetInstanceProcAddr = $func;
        let name: &str = $name;
        let raw = call_get_instance_proc_addr(instance, func, name);
        match raw {
            Ok(f) => unsafe { std::mem::transmute::<pfn::VoidFunction, $t>(f) },
            Err(error) => {
                return Err(format!(
                    "Failed to load {} for instance {} with error {}",
                    name,
                    instance.into_raw(),
                    decode_xr_result(error)
                ));
            }
        }
    }};
}

pub fn load(func: pfn::GetInstanceProcAddr) -> Result<XrFunctions, String> {
    let functions = XrFunctions {
        get_instance_proc_addr: func,
        enumerate_instance_extension_properties: find_and_cast!(
            func,
            "xrEnumerateInstanceExtensionProperties",
            pfn::EnumerateInstanceExtensionProperties
        ),
        enumerate_api_layer_properties: find_and_cast!(
            func,
            "xrEnumerateApiLayerProperties",
            pfn::EnumerateApiLayerProperties
        ),
        create_instance: find_and_cast!(func, "xrCreateInstance", pfn::CreateInstance),
    };
    return Ok(functions);
}

pub fn load_instance(
    instance: openxr_sys::Instance,
    get_instance_proc_addr: pfn::GetInstanceProcAddr,
) -> Result<XrInstanceFunctions, String> {
    if instance == openxr_sys::Instance::NULL {
        return Err("Instance must not be NULL".to_owned());
    }
    let functions = XrInstanceFunctions {
        destroy_instance: find_and_cast!(
            instance,
            get_instance_proc_addr,
            "xrDestroyInstance",
            pfn::DestroyInstance
        ),
    };
    return Ok(functions);
}

fn call_get_instance_proc_addr(
    instance: openxr_sys::Instance,
    func: pfn::GetInstanceProcAddr,
    name: &str,
) -> Result<pfn::VoidFunction, openxr_sys::Result> {
    let mut function: Option<pfn::VoidFunction> = Option::None;
    let name_cstr = CString::new(name).unwrap();
    unsafe {
        let status = func(instance, name_cstr.as_ptr(), &mut function);
        if status != openxr_sys::Result::SUCCESS {
            return Err(status);
        }
    }
    return match function {
        Some(f) => Ok(f),
        None => Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE),
    };
}

#[rustfmt::skip]
pub fn decode_xr_result(result: openxr_sys::Result) -> String {
    match result {
        openxr_sys::Result::SUCCESS => "XR_SUCCESS".to_owned(),
        openxr_sys::Result::TIMEOUT_EXPIRED => "XR_TIMEOUT_EXPIRED".to_owned(),
        openxr_sys::Result::SESSION_LOSS_PENDING => "XR_SESSION_LOSS_PENDING".to_owned(),
        openxr_sys::Result::EVENT_UNAVAILABLE => "XR_EVENT_UNAVAILABLE".to_owned(),
        openxr_sys::Result::SPACE_BOUNDS_UNAVAILABLE => "XR_SPACE_BOUNDS_UNAVAILABLE".to_owned(),
        openxr_sys::Result::SESSION_NOT_FOCUSED => "XR_SESSION_NOT_FOCUSED".to_owned(),
        openxr_sys::Result::FRAME_DISCARDED => "XR_FRAME_DISCARDED".to_owned(),
        openxr_sys::Result::ERROR_VALIDATION_FAILURE => "XR_ERROR_VALIDATION_FAILURE".to_owned(),
        openxr_sys::Result::ERROR_RUNTIME_FAILURE => "XR_ERROR_RUNTIME_FAILURE".to_owned(),
        openxr_sys::Result::ERROR_OUT_OF_MEMORY => "XR_ERROR_OUT_OF_MEMORY".to_owned(),
        openxr_sys::Result::ERROR_API_VERSION_UNSUPPORTED => "XR_ERROR_API_VERSION_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_INITIALIZATION_FAILED => "XR_ERROR_INITIALIZATION_FAILED".to_owned(),
        openxr_sys::Result::ERROR_FUNCTION_UNSUPPORTED => "XR_ERROR_FUNCTION_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_FEATURE_UNSUPPORTED => "XR_ERROR_FEATURE_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_EXTENSION_NOT_PRESENT => "XR_ERROR_EXTENSION_NOT_PRESENT".to_owned(),
        openxr_sys::Result::ERROR_LIMIT_REACHED => "XR_ERROR_LIMIT_REACHED".to_owned(),
        openxr_sys::Result::ERROR_SIZE_INSUFFICIENT => "XR_ERROR_SIZE_INSUFFICIENT".to_owned(),
        openxr_sys::Result::ERROR_HANDLE_INVALID => "XR_ERROR_HANDLE_INVALID".to_owned(),
        openxr_sys::Result::ERROR_INSTANCE_LOST => "XR_ERROR_INSTANCE_LOST".to_owned(),
        openxr_sys::Result::ERROR_SESSION_RUNNING => "XR_ERROR_SESSION_RUNNING".to_owned(),
        openxr_sys::Result::ERROR_SESSION_NOT_RUNNING => "XR_ERROR_SESSION_NOT_RUNNING".to_owned(),
        openxr_sys::Result::ERROR_SESSION_LOST => "XR_ERROR_SESSION_LOST".to_owned(),
        openxr_sys::Result::ERROR_SYSTEM_INVALID => "XR_ERROR_SYSTEM_INVALID".to_owned(),
        openxr_sys::Result::ERROR_PATH_INVALID => "XR_ERROR_PATH_INVALID".to_owned(),
        openxr_sys::Result::ERROR_PATH_COUNT_EXCEEDED => "XR_ERROR_PATH_COUNT_EXCEEDED".to_owned(),
        openxr_sys::Result::ERROR_PATH_FORMAT_INVALID => "XR_ERROR_PATH_FORMAT_INVALID".to_owned(),
        openxr_sys::Result::ERROR_PATH_UNSUPPORTED => "XR_ERROR_PATH_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_LAYER_INVALID => "XR_ERROR_LAYER_INVALID".to_owned(),
        openxr_sys::Result::ERROR_LAYER_LIMIT_EXCEEDED => "XR_ERROR_LAYER_LIMIT_EXCEEDED".to_owned(),
        openxr_sys::Result::ERROR_SWAPCHAIN_RECT_INVALID => "XR_ERROR_SWAPCHAIN_RECT_INVALID".to_owned(),
        openxr_sys::Result::ERROR_SWAPCHAIN_FORMAT_UNSUPPORTED => "XR_ERROR_SWAPCHAIN_FORMAT_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_ACTION_TYPE_MISMATCH => "XR_ERROR_ACTION_TYPE_MISMATCH".to_owned(),
        openxr_sys::Result::ERROR_SESSION_NOT_READY => "XR_ERROR_SESSION_NOT_READY".to_owned(),
        openxr_sys::Result::ERROR_SESSION_NOT_STOPPING => "XR_ERROR_SESSION_NOT_STOPPING".to_owned(),
        openxr_sys::Result::ERROR_TIME_INVALID => "XR_ERROR_TIME_INVALID".to_owned(),
        openxr_sys::Result::ERROR_REFERENCE_SPACE_UNSUPPORTED => "XR_ERROR_REFERENCE_SPACE_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_FILE_ACCESS_ERROR => "XR_ERROR_FILE_ACCESS_ERROR".to_owned(),
        openxr_sys::Result::ERROR_FILE_CONTENTS_INVALID => "XR_ERROR_FILE_CONTENTS_INVALID".to_owned(),
        openxr_sys::Result::ERROR_FORM_FACTOR_UNSUPPORTED => "XR_ERROR_FORM_FACTOR_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_FORM_FACTOR_UNAVAILABLE => "XR_ERROR_FORM_FACTOR_UNAVAILABLE".to_owned(),
        openxr_sys::Result::ERROR_API_LAYER_NOT_PRESENT => "XR_ERROR_API_LAYER_NOT_PRESENT".to_owned(),
        openxr_sys::Result::ERROR_CALL_ORDER_INVALID => "XR_ERROR_CALL_ORDER_INVALID".to_owned(),
        openxr_sys::Result::ERROR_GRAPHICS_DEVICE_INVALID => "XR_ERROR_GRAPHICS_DEVICE_INVALID".to_owned(),
        openxr_sys::Result::ERROR_POSE_INVALID => "XR_ERROR_POSE_INVALID".to_owned(),
        openxr_sys::Result::ERROR_INDEX_OUT_OF_RANGE => "XR_ERROR_INDEX_OUT_OF_RANGE".to_owned(),
        openxr_sys::Result::ERROR_VIEW_CONFIGURATION_TYPE_UNSUPPORTED => "XR_ERROR_VIEW_CONFIGURATION_TYPE_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_ENVIRONMENT_BLEND_MODE_UNSUPPORTED => "XR_ERROR_ENVIRONMENT_BLEND_MODE_UNSUPPORTED".to_owned(),
        openxr_sys::Result::ERROR_NAME_DUPLICATED => "XR_ERROR_NAME_DUPLICATED".to_owned(),
        openxr_sys::Result::ERROR_NAME_INVALID => "XR_ERROR_NAME_INVALID".to_owned(),
        openxr_sys::Result::ERROR_ACTIONSET_NOT_ATTACHED => "XR_ERROR_ACTIONSET_NOT_ATTACHED".to_owned(),
        openxr_sys::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED => "XR_ERROR_ACTIONSETS_ALREADY_ATTACHED".to_owned(),
        openxr_sys::Result::ERROR_LOCALIZED_NAME_DUPLICATED => "XR_ERROR_LOCALIZED_NAME_DUPLICATED".to_owned(),
        openxr_sys::Result::ERROR_LOCALIZED_NAME_INVALID => "XR_ERROR_LOCALIZED_NAME_INVALID".to_owned(),
        openxr_sys::Result::ERROR_GRAPHICS_REQUIREMENTS_CALL_MISSING => "XR_ERROR_GRAPHICS_REQUIREMENTS_CALL_MISSING".to_owned(),
        openxr_sys::Result::ERROR_RUNTIME_UNAVAILABLE => "XR_ERROR_RUNTIME_UNAVAILABLE".to_owned(),
        openxr_sys::Result::ERROR_ANDROID_THREAD_SETTINGS_ID_INVALID_KHR => "XR_ERROR_ANDROID_THREAD_SETTINGS_ID_INVALID_KHR".to_owned(),
        openxr_sys::Result::ERROR_ANDROID_THREAD_SETTINGS_FAILURE_KHR => "XR_ERROR_ANDROID_THREAD_SETTINGS_FAILURE_KHR".to_owned(),
        openxr_sys::Result::ERROR_CREATE_SPATIAL_ANCHOR_FAILED_MSFT => "XR_ERROR_CREATE_SPATIAL_ANCHOR_FAILED_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SECONDARY_VIEW_CONFIGURATION_TYPE_NOT_ENABLED_MSFT => "XR_ERROR_SECONDARY_VIEW_CONFIGURATION_TYPE_NOT_ENABLED_MSFT".to_owned(),
        openxr_sys::Result::ERROR_CONTROLLER_MODEL_KEY_INVALID_MSFT => "XR_ERROR_CONTROLLER_MODEL_KEY_INVALID_MSFT".to_owned(),
        openxr_sys::Result::ERROR_REPROJECTION_MODE_UNSUPPORTED_MSFT => "XR_ERROR_REPROJECTION_MODE_UNSUPPORTED_MSFT".to_owned(),
        openxr_sys::Result::ERROR_COMPUTE_NEW_SCENE_NOT_COMPLETED_MSFT => "XR_ERROR_COMPUTE_NEW_SCENE_NOT_COMPLETED_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SCENE_COMPONENT_ID_INVALID_MSFT => "XR_ERROR_SCENE_COMPONENT_ID_INVALID_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SCENE_COMPONENT_TYPE_MISMATCH_MSFT => "XR_ERROR_SCENE_COMPONENT_TYPE_MISMATCH_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SCENE_MESH_BUFFER_ID_INVALID_MSFT => "XR_ERROR_SCENE_MESH_BUFFER_ID_INVALID_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SCENE_COMPUTE_FEATURE_INCOMPATIBLE_MSFT => "XR_ERROR_SCENE_COMPUTE_FEATURE_INCOMPATIBLE_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SCENE_COMPUTE_CONSISTENCY_MISMATCH_MSFT => "XR_ERROR_SCENE_COMPUTE_CONSISTENCY_MISMATCH_MSFT".to_owned(),
        openxr_sys::Result::ERROR_DISPLAY_REFRESH_RATE_UNSUPPORTED_FB => "XR_ERROR_DISPLAY_REFRESH_RATE_UNSUPPORTED_FB".to_owned(),
        openxr_sys::Result::ERROR_COLOR_SPACE_UNSUPPORTED_FB => "XR_ERROR_COLOR_SPACE_UNSUPPORTED_FB".to_owned(),
        openxr_sys::Result::ERROR_UNEXPECTED_STATE_PASSTHROUGH_FB => "XR_ERROR_UNEXPECTED_STATE_PASSTHROUGH_FB".to_owned(),
        openxr_sys::Result::ERROR_FEATURE_ALREADY_CREATED_PASSTHROUGH_FB => "XR_ERROR_FEATURE_ALREADY_CREATED_PASSTHROUGH_FB".to_owned(),
        openxr_sys::Result::ERROR_FEATURE_REQUIRED_PASSTHROUGH_FB => "XR_ERROR_FEATURE_REQUIRED_PASSTHROUGH_FB".to_owned(),
        openxr_sys::Result::ERROR_NOT_PERMITTED_PASSTHROUGH_FB => "XR_ERROR_NOT_PERMITTED_PASSTHROUGH_FB".to_owned(),
        openxr_sys::Result::ERROR_INSUFFICIENT_RESOURCES_PASSTHROUGH_FB => "XR_ERROR_INSUFFICIENT_RESOURCES_PASSTHROUGH_FB".to_owned(),
        openxr_sys::Result::ERROR_UNKNOWN_PASSTHROUGH_FB => "XR_ERROR_UNKNOWN_PASSTHROUGH_FB".to_owned(),
        openxr_sys::Result::ERROR_MARKER_NOT_TRACKED_VARJO => "XR_ERROR_MARKER_NOT_TRACKED_VARJO".to_owned(),
        openxr_sys::Result::ERROR_MARKER_ID_INVALID_VARJO => "XR_ERROR_MARKER_ID_INVALID_VARJO".to_owned(),
        openxr_sys::Result::ERROR_SPATIAL_ANCHOR_NAME_NOT_FOUND_MSFT => "XR_ERROR_SPATIAL_ANCHOR_NAME_NOT_FOUND_MSFT".to_owned(),
        openxr_sys::Result::ERROR_SPATIAL_ANCHOR_NAME_INVALID_MSFT => "XR_ERROR_SPATIAL_ANCHOR_NAME_INVALID_MSFT".to_owned(),
        default => format!("XR_UNKNOWN_{}_{}", if default.into_raw() < 0 {"SUCCESS"} else { "FAILURE" }, default),
    }
}
