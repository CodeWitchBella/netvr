use std::{
    ffi::CStr,
    fmt,
    fmt::{Debug, Pointer},
    os::raw::c_char,
};

use crate::{SizedArrayValueIterator, XrDebug};

#[derive(Clone)]
pub struct XrStruct {
    pub ty: openxr_sys::StructureType,
    data: *const openxr_sys::BaseInStructure,
}

macro_rules! implement_struct {
    ($($id: ident), *,) => {
        $(
            #[repr(transparent)]
            pub struct $id<'a>(pub(crate) &'a openxr_sys::$id);

            impl<'a> $id<'a> {
                #[inline]
                pub fn as_raw(&'a self) -> &'a openxr_sys::$id {
                    self.0
                }
            }

            impl<'a> std::fmt::Debug for $id<'a> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }
        )*

        impl XrDebug for XrStruct {
            fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
                match self.ty {
                    $(
                        openxr_sys::$id::TYPE => {
                            $id(unsafe {
                                &*std::mem::transmute::<
                                    *const openxr_sys::BaseInStructure,
                                    *const openxr_sys::$id,
                                >(self.data)
                            }).xr_fmt(f, instance)
                        }
                    )*
                    _ => f
                        .debug_struct("Unknown")
                        .field("ty", &self.ty)
                        .finish_non_exhaustive(),
                }
            }
        }
    };
}

implement_struct!(
    // Following are missing because they are android-only and I did not want to
    // spend time to try and figure out how to integrate them, since I do not
    // need them (for now).
    //   - read_android_surface_swapchain_create_info_fb reads AndroidSurfaceSwapchainCreateInfoFB,
    //   - read_graphics_binding_open_gles_android_khr reads GraphicsBindingOpenGLESAndroidKHR,
    //   - read_instance_create_info_android_khr reads InstanceCreateInfoAndroidKHR,
    //   - read_loader_init_info_android_khr reads LoaderInitInfoAndroidKHR,
    // In the same vein, following are win32-only
    //   - read_graphics_binding_d3_d11_khr reads GraphicsBindingD3D11KHR,
    //   - read_graphics_binding_d3_d12_khr reads GraphicsBindingD3D12KHR,
    //   - read_graphics_binding_open_gl_win32_khr reads GraphicsBindingOpenGLWin32KHR,
    //   - read_holographic_window_attachment_msft reads HolographicWindowAttachmentMSFT,
    // And linux only
    //   - read_graphics_binding_open_gl_xcb_khr reads GraphicsBindingOpenGLXcbKHR,
    ActionCreateInfo,
    ActionSetCreateInfo,
    ActionSpaceCreateInfo,
    ActionStateGetInfo,
    ActionsSyncInfo,
    BindingModificationsKHR,
    BoundSourcesForActionEnumerateInfo,
    CompositionLayerColorScaleBiasKHR,
    CompositionLayerCubeKHR,
    CompositionLayerCylinderKHR,
    CompositionLayerDepthInfoKHR,
    CompositionLayerDepthTestVARJO,
    CompositionLayerEquirectKHR,
    CompositionLayerEquirect2KHR,
    CompositionLayerPassthroughFB,
    CompositionLayerProjection,
    CompositionLayerProjectionView,
    CompositionLayerQuad,
    CompositionLayerReprojectionInfoMSFT,
    CompositionLayerReprojectionPlaneOverrideMSFT,
    CompositionLayerSecureContentFB,
    CompositionLayerSpaceWarpInfoFB,
    DebugUtilsLabelEXT,
    DebugUtilsMessengerCallbackDataEXT,
    DebugUtilsMessengerCreateInfoEXT,
    DebugUtilsObjectNameInfoEXT,
    DigitalLensControlALMALENCE,
    EventDataBuffer,
    EventDataDisplayRefreshRateChangedFB,
    EventDataEventsLost,
    EventDataInstanceLossPending,
    EventDataInteractionProfileChanged,
    EventDataMainSessionVisibilityChangedEXTX,
    EventDataMarkerTrackingUpdateVARJO,
    EventDataPassthroughStateChangedFB,
    EventDataPerfSettingsEXT,
    EventDataReferenceSpaceChangePending,
    EventDataSessionStateChanged,
    EventDataVisibilityMaskChangedKHR,
    EventDataViveTrackerConnectedHTCX,
    FacialExpressionsHTC,
    FacialTrackerCreateInfoHTC,
    FrameEndInfo,
    GeometryInstanceCreateInfoFB,
    GeometryInstanceTransformFB,
    GraphicsBindingEGLMNDX,
    GraphicsBindingOpenGLWaylandKHR,
    GraphicsBindingOpenGLXlibKHR,
    GraphicsBindingVulkanKHR,
    HandJointsLocateInfoEXT,
    HandJointsMotionRangeInfoEXT,
    HandMeshSpaceCreateInfoMSFT,
    HandMeshUpdateInfoMSFT,
    HandPoseTypeInfoMSFT,
    HandTrackerCreateInfoEXT,
    HapticActionInfo,
    HapticVibration,
    InputSourceLocalizedNameGetInfo,
    InstanceCreateInfo,
    InteractionProfileAnalogThresholdVALVE,
    InteractionProfileSuggestedBinding,
    MarkerSpaceCreateInfoVARJO,
    PassthroughColorMapMonoToMonoFB,
    PassthroughColorMapMonoToRgbaFB,
    PassthroughCreateInfoFB,
    PassthroughKeyboardHandsIntensityFB,
    PassthroughLayerCreateInfoFB,
    PassthroughStyleFB,
    ReferenceSpaceCreateInfo,
    SecondaryViewConfigurationFrameEndInfoMSFT,
    SecondaryViewConfigurationLayerInfoMSFT,
    SecondaryViewConfigurationSessionBeginInfoMSFT,
    SecondaryViewConfigurationSwapchainCreateInfoMSFT,
    SessionActionSetsAttachInfo,
    SessionBeginInfo,
    SessionCreateInfo,
    SessionCreateInfoOverlayEXTX,
    SpatialAnchorCreateInfoMSFT,
    SpatialAnchorFromPersistedAnchorCreateInfoMSFT,
    SpatialAnchorPersistenceInfoMSFT,
    SpatialAnchorSpaceCreateInfoMSFT,
    SpatialGraphNodeSpaceCreateInfoMSFT,
    SwapchainCreateInfo,
    SwapchainImageWaitInfo,
    SystemGetInfo,
    SystemPassthroughPropertiesFB,
    TriangleMeshCreateInfoFB,
    ViewConfigurationViewFovEPIC,
    ViewLocateFoveatedRenderingVARJO,
    ViewLocateInfo,
    VulkanDeviceCreateInfoKHR,
    VulkanGraphicsDeviceGetInfoKHR,
    VulkanInstanceCreateInfoKHR,
    VulkanSwapchainFormatListCreateInfoKHR,
    // I somehow missed this one in first version, which means that there likely
    // are more missing from the list.
    InteractionProfileState,
);

impl XrStruct {
    pub(crate) fn from(arg: *const openxr_sys::BaseInStructure) -> Self {
        let ty = unsafe { *arg }.ty;

        Self { ty, data: arg }
    }
}

#[derive(Debug)]
pub enum StringParseError {
    NotNullTerminated,
    Utf8Error(std::str::Utf8Error),
}

fn parse_input_string(name_ptr: &[c_char]) -> Result<&str, StringParseError> {
    if name_ptr[name_ptr.len() - 1] != 0 {
        return Err(StringParseError::NotNullTerminated);
    };
    match unsafe { CStr::from_ptr(name_ptr.as_ptr()) }.to_str() {
        Ok(val) => Ok(val),
        Err(error) => Err(StringParseError::Utf8Error(error)),
    }
}

impl<'a> ActionCreateInfo<'a> {
    pub fn action_name(&'a self) -> Result<&'a str, StringParseError> {
        parse_input_string(&self.0.action_name)
    }

    pub fn localized_action_name(&'a self) -> Result<&'a str, StringParseError> {
        parse_input_string(&self.0.localized_action_name)
    }

    pub fn action_type(&self) -> openxr_sys::ActionType {
        self.0.action_type
    }

    pub fn subaction_paths(&self) -> SizedArrayValueIterator<openxr_sys::Path> {
        unsafe {
            SizedArrayValueIterator::new(self.0.count_subaction_paths, self.0.subaction_paths)
        }
    }
}

impl<'a> ActionsSyncInfo<'a> {
    pub fn active_action_sets(&self) -> SizedArrayValueIterator<openxr_sys::ActiveActionSet> {
        unsafe {
            SizedArrayValueIterator::new(self.0.count_active_action_sets, self.0.active_action_sets)
        }
    }
}

impl<'a> ActionStateGetInfo<'a> {
    pub fn subaction_path(&self) -> openxr::Path {
        self.0.subaction_path
    }
    pub fn action(&self) -> openxr_sys::Action {
        self.0.action
    }
}

impl<'a> EventDataInteractionProfileChanged<'a> {
    pub fn session(&self) -> openxr_sys::Session {
        self.0.session
    }
}

impl<'a> InteractionProfileState<'a> {
    pub fn interaction_profile(&self) -> openxr_sys::Path {
        self.0.interaction_profile
    }
}

impl<'a> InteractionProfileSuggestedBinding<'a> {
    pub fn interaction_profile(&self) -> openxr::Path {
        self.0.interaction_profile
    }

    pub fn suggested_bindings(
        &self,
    ) -> SizedArrayValueIterator<openxr_sys::ActionSuggestedBinding> {
        unsafe {
            SizedArrayValueIterator::new(self.0.count_suggested_bindings, self.0.suggested_bindings)
        }
    }
}
