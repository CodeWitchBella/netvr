use std::{borrow::BorrowMut, ffi::CStr, fmt, num::TryFromIntError, str::Utf8Error};

use crate::{
    utils::ResultConvertible,
    xr_struct::{self, ActionCreateInfo},
};

pub struct XrDebugValue<'a, T: XrDebug>(pub(crate) openxr::Instance, pub(crate) &'a T);

impl<'a, T: XrDebug> fmt::Debug for XrDebugValue<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.xr_fmt(f, &self.0)
    }
}

/// Allows object to be debugged with extra information obtained from OpenXR
/// runtime.
pub trait XrDebug {
    /// Acts similarly to std::fmt::Debug::fmt but may call OpenXR function to
    /// reveal further detail about given object.
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result;

    /// This is usually how you consume object which implements XrDebug trait.
    ///
    /// If conversion to string and/or printing the object is desired
    /// ```ignore
    /// format!("{:?}", object.as_debug(&self.instance))
    /// ```
    ///
    /// Or if you are implementing Debug or XrDebug (usually depending on the
    /// availability of openxr::Instance reference)
    ///
    /// ```ignore
    /// f.field("field", &self.field.as_debug(instance))
    /// ```
    fn as_debug(&self, instance: &openxr::Instance) -> XrDebugValue<Self>
    where
        Self: std::marker::Sized + XrDebug,
    {
        XrDebugValue(instance.clone(), self)
    }
}

impl<T> XrDebug for &T
where
    T: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        (*self).xr_fmt(f, instance)
    }
}

impl<T> XrDebug for Option<T>
where
    T: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        f.debug_struct("TODO::Option<T>").finish()
    }
}

impl<K, V> XrDebug for std::collections::HashMap<K, V>
where
    K: XrDebug,
    V: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let mut deb = f.debug_map();
        for (k, v) in self.iter() {
            deb.entry(&k.as_debug(instance), &v.as_debug(instance));
        }
        deb.finish()
    }
}

impl<V> XrDebug for Vec<V>
where
    V: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let mut deb = f.debug_list();
        for v in self.iter() {
            deb.entry(&v.as_debug(instance));
        }
        deb.finish()
    }
}

impl<T> XrDebug for Result<T, openxr_sys::Result>
where
    T: XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        match self {
            Ok(val) => f.debug_tuple("Ok").field(&val.as_debug(instance)).finish(),
            Err(err) => f.debug_tuple("Err").field(err).finish(),
        }
    }
}

impl XrDebug for ActionCreateInfo<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
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
    }
}

impl XrDebug for crate::xr_struct::EventDataSessionStateChanged<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let raw = self.as_raw();
        f.debug_struct("EventDataSessionStateChanged")
            .field("session", &raw.session)
            .field("state", &raw.state)
            .field("time", &XrDebugValue(instance.clone(), &raw.time))
            .finish()
    }
}

impl XrDebug for openxr_sys::Time {
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        let mut f = f.debug_struct("Time");
        let mut value = self.as_nanos();
        f.field("raw", &value);
        f.field("ns", &(value % 1000));
        value /= 1000;
        if value < 1 {
            return f.finish();
        }
        f.field("us", &(value % 1000));
        value /= 1000;
        if value < 1 {
            return f.finish();
        }
        f.field("ms", &(value % 1000));
        value /= 1000;
        if value < 1 {
            return f.finish();
        }
        f.field("s", &(value % 60));
        value /= 60;
        if value < 1 {
            return f.finish();
        }
        f.field("min", &(value % 60));
        value /= 60;
        if value < 1 {
            return f.finish();
        }
        f.field("h", &(value % 24));
        value /= 24;
        if value < 1 {
            return f.finish();
        }
        f.field("d", &value);
        f.finish()
    }
}

impl XrDebug for xr_struct::ActionsSyncInfo<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActionsSyncInfo")
            .field(
                "active_action_sets",
                &self.active_action_sets().as_debug(instance),
            )
            .finish()
    }
}

impl XrDebug for xr_struct::SessionCreateInfo<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, _instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("SessionCreateInfo").finish_non_exhaustive()
    }
}

impl XrDebug for openxr_sys::ActiveActionSet {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActiveActionSet")
            .field("action_set", &self.action_set.as_debug(instance))
            .field("subaction_path", &self.subaction_path.as_debug(instance))
            .finish()
    }
}

impl XrDebug for openxr_sys::ActionSuggestedBinding {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActionSuggestedBinding")
            .field("action", &self.action.as_debug(instance))
            .field("binding", &self.binding.as_debug(instance))
            .finish()
    }
}

#[derive(Debug)]
enum DebugPathError {
    FromBytesWithNul(std::ffi::FromBytesWithNulError),
    Utf8(Utf8Error),
    GetPathLength(openxr_sys::Result),
    GetPathString(openxr_sys::Result),
    TryFromInt(TryFromIntError),
}

/// Utility function which takes openxr_sys::Path and prints it with formatter.
/// If something goes wrong it returns empty error ().
fn debug_path(
    path: &openxr_sys::Path,
    f: &mut fmt::Formatter,
    instance: &openxr::Instance,
) -> Result<fmt::Result, DebugPathError> {
    if path.into_raw() == 0 {
        // steamvr returns XR_RUNTIME_FAILURE for null paths
        return Ok(f.debug_tuple("Path").field(&"null").finish());
    }

    let mut size_u32: u32 = 0;

    unsafe {
        (instance.fp().path_to_string)(
            instance.as_raw(),
            *path,
            0,
            size_u32.borrow_mut(),
            std::ptr::null_mut(),
        )
        .into_result()
        .map_err(DebugPathError::GetPathLength)?;
        let size: usize = size_u32.try_into().map_err(DebugPathError::TryFromInt)?;

        let mut vec = vec![0_u8; size];
        (instance.fp().path_to_string)(
            instance.as_raw(),
            *path,
            size_u32,
            size_u32.borrow_mut(),
            std::mem::transmute(vec.as_mut_ptr()),
        )
        .into_result()
        .map_err(DebugPathError::GetPathString)?;

        let str = CStr::from_bytes_with_nul(vec.as_slice())
            .map_err(DebugPathError::FromBytesWithNul)?
            .to_str()
            .map_err(DebugPathError::Utf8)?;
        Ok(f.debug_tuple("Path").field(&str).finish())
    }
}

impl XrDebug for openxr_sys::Path {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        match debug_path(self, f, instance) {
            Ok(result) => result,
            Err(err) => f
                .debug_tuple("Path")
                .field(&format!("Invalid({}, {:?})", self.into_raw(), err))
                .finish(),
        }
    }
}

impl XrDebug for openxr_sys::Posef {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("Posef")
            .field("position", &self.position.as_debug(instance))
            .field("orientation", &self.orientation.as_debug(instance))
            .finish()
    }
}

impl XrDebug for openxr_sys::Vector3f {
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        f.debug_struct("Vector3f")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}

impl XrDebug for openxr_sys::Quaternionf {
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        f.debug_struct("Quaternionf")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .field("w", &self.w)
            .finish()
    }
}

impl XrDebug for xr_struct::ActionStateGetInfo<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActionStateGetInfo")
            .field("subaction_path", &self.subaction_path().as_debug(instance))
            .field("action", &self.action().as_debug(instance))
            .finish()
    }
}

impl XrDebug for xr_struct::EventDataInteractionProfileChanged<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("EventDataInteractionProfileChanged")
            .field("session", &self.session().as_debug(instance))
            .finish()
    }
}

impl XrDebug for xr_struct::InteractionProfileState<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("InteractionProfileState")
            .field(
                "interaction_profile",
                &self.interaction_profile().as_debug(instance),
            )
            .finish()
    }
}

impl XrDebug for xr_struct::ActionStateBoolean<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        self.0.xr_fmt(f, instance)
    }
}

impl XrDebug for xr_struct::InteractionProfileSuggestedBinding<'_> {
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        f.debug_struct("InteractionProfileSuggestedBinding")
            .field(
                "interaction_profile",
                &self.interaction_profile().as_debug(instance),
            )
            .field(
                "suggested_bindings",
                &self.suggested_bindings().as_debug(instance),
            )
            .finish()
    }
}

macro_rules! implement_as_action_state {
    ($id:ident) => {
        impl XrDebug for openxr_sys::$id {
            fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
                f.debug_struct(stringify!($id))
                    .field("changed_since_last_sync", &self.changed_since_last_sync)
                    .field("current_state", &self.current_state)
                    .field("is_active", &self.is_active)
                    .field("last_change_time", &self.last_change_time)
                    .finish()
            }
        }
    };
}
implement_as_action_state!(ActionStateBoolean);
implement_as_action_state!(ActionStateFloat);
implement_as_action_state!(ActionStateVector2f);

impl XrDebug for openxr_sys::ActionStatePose {
    fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
        f.debug_struct("ActionStatePose")
            .field("changed_since_last_sync", &self.is_active)
            .field("is_active", &self.is_active)
            .finish()
        // Pose does not have last_change_time nor current_state since it uses
        // XrSpace instead
    }
}

macro_rules! implement_as_handle {
    ($($id: ident), *,) => {
        $(
            impl XrDebug for openxr_sys::$id {
                fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
                    f.write_fmt(format_args!("{}({})", stringify!($id), self.into_raw()))
                }
            }
        )*
    };
}

implement_as_handle!(
    Instance,
    Session,
    Swapchain,
    Space,
    ActionSet,
    Action,
    DebugUtilsMessengerEXT,
    SpatialAnchorMSFT,
    SpatialGraphNodeTypeMSFT,
    HandTrackerEXT,
    SceneObserverMSFT,
    SceneMSFT,
    FacialTrackerHTC,
    FoveationProfileFB,
    TriangleMeshFB,
    PassthroughFB,
    PassthroughLayerFB,
    GeometryInstanceFB,
    SpatialAnchorStoreConnectionMSFT,
);

macro_rules! implement_as_non_exhaustive {
    ($($id: ty), *,) => {
        $(
            impl XrDebug for $id {
                fn xr_fmt(&self, f: &mut fmt::Formatter, _: &openxr::Instance) -> fmt::Result {
                    f.debug_struct(stringify!($id)).finish_non_exhaustive()
                }
            }
        )*
    };
}

implement_as_non_exhaustive!(
    // Following are missing because they are android-only and I did not want to
    // spend time to try and figure out how to integrate them, since I do not
    // need them (for now).
    //   - xr_struct::AndroidSurfaceSwapchainCreateInfoFB<'_>,
    //   - xr_struct::GraphicsBindingOpenGLESAndroidKHR<'_>,
    //   - xr_struct::InstanceCreateInfoAndroidKHR<'_>,
    //   - xr_struct::LoaderInitInfoAndroidKHR<'_>,
    //
    // Following types have hand-written xr_fmt implementations
    //   - xr_struct::ActionCreateInfo<'_>,
    //   - xr_struct::ActionsSyncInfo<'_>,
    //   - xr_struct::EventDataSessionStateChanged<'_>,
    //   - xr_struct::ActionStateGetInfo<'_>,
    //   - xr_struct::SessionCreateInfo<'_>,
    //   - xr_struct::EventDataInteractionProfileChanged<'_>,
    //   - xr_struct::InteractionProfileState<'_>,
    //
    // Following types are readable via XrStructChain and do not have full implementation
    xr_struct::ActionSetCreateInfo<'_>,
    xr_struct::ActionSpaceCreateInfo<'_>,
    xr_struct::BindingModificationsKHR<'_>,
    xr_struct::BoundSourcesForActionEnumerateInfo<'_>,
    xr_struct::CompositionLayerColorScaleBiasKHR<'_>,
    xr_struct::CompositionLayerCubeKHR<'_>,
    xr_struct::CompositionLayerCylinderKHR<'_>,
    xr_struct::CompositionLayerDepthInfoKHR<'_>,
    xr_struct::CompositionLayerDepthTestVARJO<'_>,
    xr_struct::CompositionLayerEquirect2KHR<'_>,
    xr_struct::CompositionLayerEquirectKHR<'_>,
    xr_struct::CompositionLayerPassthroughFB<'_>,
    xr_struct::CompositionLayerProjection<'_>,
    xr_struct::CompositionLayerProjectionView<'_>,
    xr_struct::CompositionLayerQuad<'_>,
    xr_struct::CompositionLayerReprojectionInfoMSFT<'_>,
    xr_struct::CompositionLayerReprojectionPlaneOverrideMSFT<'_>,
    xr_struct::CompositionLayerSecureContentFB<'_>,
    xr_struct::CompositionLayerSpaceWarpInfoFB<'_>,
    xr_struct::DebugUtilsLabelEXT<'_>,
    xr_struct::DebugUtilsMessengerCallbackDataEXT<'_>,
    xr_struct::DebugUtilsMessengerCreateInfoEXT<'_>,
    xr_struct::DebugUtilsObjectNameInfoEXT<'_>,
    xr_struct::DigitalLensControlALMALENCE<'_>,
    xr_struct::EventDataBuffer<'_>,
    xr_struct::EventDataDisplayRefreshRateChangedFB<'_>,
    xr_struct::EventDataEventsLost<'_>,
    xr_struct::EventDataInstanceLossPending<'_>,
    xr_struct::EventDataMainSessionVisibilityChangedEXTX<'_>,
    xr_struct::EventDataMarkerTrackingUpdateVARJO<'_>,
    xr_struct::EventDataPassthroughStateChangedFB<'_>,
    xr_struct::EventDataPerfSettingsEXT<'_>,
    xr_struct::EventDataReferenceSpaceChangePending<'_>,
    xr_struct::EventDataVisibilityMaskChangedKHR<'_>,
    xr_struct::EventDataViveTrackerConnectedHTCX<'_>,
    xr_struct::FacialExpressionsHTC<'_>,
    xr_struct::FacialTrackerCreateInfoHTC<'_>,
    xr_struct::FrameEndInfo<'_>,
    xr_struct::GeometryInstanceCreateInfoFB<'_>,
    xr_struct::GeometryInstanceTransformFB<'_>,
    // xr_struct::GraphicsBindingD3D11KHR<'_>,
    // xr_struct::GraphicsBindingD3D12KHR<'_>,
    xr_struct::GraphicsBindingEGLMNDX<'_>,
    xr_struct::GraphicsBindingOpenGLWaylandKHR<'_>,
    // xr_struct::GraphicsBindingOpenGLWin32KHR<'_>,
    // xr_struct::GraphicsBindingOpenGLXcbKHR<'_>,
    xr_struct::GraphicsBindingOpenGLXlibKHR<'_>,
    xr_struct::GraphicsBindingVulkanKHR<'_>,
    xr_struct::HandJointsLocateInfoEXT<'_>,
    xr_struct::HandJointsMotionRangeInfoEXT<'_>,
    xr_struct::HandMeshSpaceCreateInfoMSFT<'_>,
    xr_struct::HandMeshUpdateInfoMSFT<'_>,
    xr_struct::HandPoseTypeInfoMSFT<'_>,
    xr_struct::HandTrackerCreateInfoEXT<'_>,
    xr_struct::HapticActionInfo<'_>,
    xr_struct::HapticVibration<'_>,
    // xr_struct::HolographicWindowAttachmentMSFT<'_>,
    xr_struct::InputSourceLocalizedNameGetInfo<'_>,
    xr_struct::InstanceCreateInfo<'_>,
    xr_struct::InteractionProfileAnalogThresholdVALVE<'_>,
    xr_struct::MarkerSpaceCreateInfoVARJO<'_>,
    xr_struct::PassthroughColorMapMonoToMonoFB<'_>,
    xr_struct::PassthroughColorMapMonoToRgbaFB<'_>,
    xr_struct::PassthroughCreateInfoFB<'_>,
    xr_struct::PassthroughKeyboardHandsIntensityFB<'_>,
    xr_struct::PassthroughLayerCreateInfoFB<'_>,
    xr_struct::PassthroughStyleFB<'_>,
    xr_struct::ReferenceSpaceCreateInfo<'_>,
    xr_struct::SecondaryViewConfigurationFrameEndInfoMSFT<'_>,
    xr_struct::SecondaryViewConfigurationLayerInfoMSFT<'_>,
    xr_struct::SecondaryViewConfigurationSessionBeginInfoMSFT<'_>,
    xr_struct::SecondaryViewConfigurationSwapchainCreateInfoMSFT<'_>,
    xr_struct::SessionActionSetsAttachInfo<'_>,
    xr_struct::SessionBeginInfo<'_>,
    xr_struct::SessionCreateInfoOverlayEXTX<'_>,
    xr_struct::SpatialAnchorCreateInfoMSFT<'_>,
    xr_struct::SpatialAnchorFromPersistedAnchorCreateInfoMSFT<'_>,
    xr_struct::SpatialAnchorPersistenceInfoMSFT<'_>,
    xr_struct::SpatialAnchorSpaceCreateInfoMSFT<'_>,
    xr_struct::SpatialGraphNodeSpaceCreateInfoMSFT<'_>,
    xr_struct::SwapchainCreateInfo<'_>,
    xr_struct::SwapchainImageWaitInfo<'_>,
    xr_struct::SystemGetInfo<'_>,
    xr_struct::SystemPassthroughPropertiesFB<'_>,
    xr_struct::TriangleMeshCreateInfoFB<'_>,
    xr_struct::ViewConfigurationViewFovEPIC<'_>,
    xr_struct::ViewLocateFoveatedRenderingVARJO<'_>,
    xr_struct::ViewLocateInfo<'_>,
    xr_struct::VulkanDeviceCreateInfoKHR<'_>,
    xr_struct::VulkanGraphicsDeviceGetInfoKHR<'_>,
    xr_struct::VulkanInstanceCreateInfoKHR<'_>,
    xr_struct::VulkanSwapchainFormatListCreateInfoKHR<'_>,
);
