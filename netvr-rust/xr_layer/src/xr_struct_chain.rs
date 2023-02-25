use std::fmt;

use crate::{xr_struct::XrStruct, XrDebug, XrDebugValue};

#[derive(Clone)]
pub struct XrStructChain {
    ptr: *const openxr_sys::BaseInStructure,
}

impl XrStructChain {
    unsafe fn new(ptr: &openxr_sys::BaseInStructure) -> Self {
        Self { ptr }
    }

    /// .
    ///
    /// # Safety
    ///
    /// You must make sure that the clone does not outlive its parent because it
    /// is actually a reference in a trench coat.
    pub(crate) unsafe fn unsafe_clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

pub trait UnsafeFrom<T> {
    /// .
    ///
    /// # Safety
    ///
    /// ptr must be valid and point to structure conforming to OpenXR structure
    /// definition (type, next, ...)
    unsafe fn from_ptr(ptr: T) -> Self;
}

macro_rules! implement_from {
    ($t: ty) => {
        impl UnsafeFrom<*const $t> for XrStructChain {
            unsafe fn from_ptr(input: *const $t) -> Self {
                XrStructChain::new(&*(input as *const openxr_sys::BaseInStructure))
            }
        }
        impl UnsafeFrom<*mut $t> for XrStructChain {
            unsafe fn from_ptr(input: *mut $t) -> Self {
                XrStructChain::new(&*(input as *const openxr_sys::BaseInStructure))
            }
        }
    };
}

// Following are missing because they are android-only and I did not want to
// spend time to try and figure out how to integrate them, since I do not
// need them (for now).
//   - implement_from!(openxr_sys::AndroidSurfaceSwapchainCreateInfoFB);
//   - implement_from!(openxr_sys::GraphicsBindingOpenGLESAndroidKHR);
//   - implement_from!(openxr_sys::InstanceCreateInfoAndroidKHR);
//   - implement_from!(openxr_sys::LoaderInitInfoAndroidKHR);
//
// List of openxr_sys types which can be converted into XrStructChain
implement_from!(openxr_sys::ActionCreateInfo);
implement_from!(openxr_sys::ActionSetCreateInfo);
implement_from!(openxr_sys::ActionSpaceCreateInfo);
implement_from!(openxr_sys::ActionsSyncInfo);
implement_from!(openxr_sys::ActionStateGetInfo);
implement_from!(openxr_sys::BindingModificationsKHR);
implement_from!(openxr_sys::BoundSourcesForActionEnumerateInfo);
implement_from!(openxr_sys::CompositionLayerColorScaleBiasKHR);
implement_from!(openxr_sys::CompositionLayerCubeKHR);
implement_from!(openxr_sys::CompositionLayerCylinderKHR);
implement_from!(openxr_sys::CompositionLayerDepthInfoKHR);
implement_from!(openxr_sys::CompositionLayerDepthTestVARJO);
implement_from!(openxr_sys::CompositionLayerEquirect2KHR);
implement_from!(openxr_sys::CompositionLayerEquirectKHR);
implement_from!(openxr_sys::CompositionLayerPassthroughFB);
implement_from!(openxr_sys::CompositionLayerProjection);
implement_from!(openxr_sys::CompositionLayerProjectionView);
implement_from!(openxr_sys::CompositionLayerQuad);
implement_from!(openxr_sys::CompositionLayerReprojectionInfoMSFT);
implement_from!(openxr_sys::CompositionLayerReprojectionPlaneOverrideMSFT);
implement_from!(openxr_sys::CompositionLayerSecureContentFB);
implement_from!(openxr_sys::CompositionLayerSpaceWarpInfoFB);
implement_from!(openxr_sys::DebugUtilsLabelEXT);
implement_from!(openxr_sys::DebugUtilsMessengerCallbackDataEXT);
implement_from!(openxr_sys::DebugUtilsMessengerCreateInfoEXT);
implement_from!(openxr_sys::DebugUtilsObjectNameInfoEXT);
implement_from!(openxr_sys::DigitalLensControlALMALENCE);
implement_from!(openxr_sys::EventDataBuffer);
implement_from!(openxr_sys::EventDataDisplayRefreshRateChangedFB);
implement_from!(openxr_sys::EventDataEventsLost);
implement_from!(openxr_sys::EventDataInstanceLossPending);
implement_from!(openxr_sys::EventDataInteractionProfileChanged);
implement_from!(openxr_sys::EventDataMainSessionVisibilityChangedEXTX);
implement_from!(openxr_sys::EventDataMarkerTrackingUpdateVARJO);
implement_from!(openxr_sys::EventDataPassthroughStateChangedFB);
implement_from!(openxr_sys::EventDataPerfSettingsEXT);
implement_from!(openxr_sys::EventDataReferenceSpaceChangePending);
implement_from!(openxr_sys::EventDataSessionStateChanged);
implement_from!(openxr_sys::EventDataVisibilityMaskChangedKHR);
implement_from!(openxr_sys::EventDataViveTrackerConnectedHTCX);
implement_from!(openxr_sys::FacialExpressionsHTC);
implement_from!(openxr_sys::FacialTrackerCreateInfoHTC);
implement_from!(openxr_sys::FrameEndInfo);
implement_from!(openxr_sys::GeometryInstanceCreateInfoFB);
implement_from!(openxr_sys::GeometryInstanceTransformFB);
//implement_from!(openxr_sys::GraphicsBindingD3D11KHR);
//implement_from!(openxr_sys::GraphicsBindingD3D12KHR);
implement_from!(openxr_sys::GraphicsBindingEGLMNDX);
implement_from!(openxr_sys::GraphicsBindingOpenGLWaylandKHR);
//implement_from!(openxr_sys::GraphicsBindingOpenGLWin32KHR);
implement_from!(openxr_sys::GraphicsBindingOpenGLXcbKHR);
implement_from!(openxr_sys::GraphicsBindingOpenGLXlibKHR);
implement_from!(openxr_sys::GraphicsBindingVulkanKHR);
implement_from!(openxr_sys::HandJointsLocateInfoEXT);
implement_from!(openxr_sys::HandJointsMotionRangeInfoEXT);
implement_from!(openxr_sys::HandMeshSpaceCreateInfoMSFT);
implement_from!(openxr_sys::HandMeshUpdateInfoMSFT);
implement_from!(openxr_sys::HandPoseTypeInfoMSFT);
implement_from!(openxr_sys::HandTrackerCreateInfoEXT);
implement_from!(openxr_sys::HapticActionInfo);
implement_from!(openxr_sys::HapticVibration);
//implement_from!(openxr_sys::HolographicWindowAttachmentMSFT);
implement_from!(openxr_sys::InputSourceLocalizedNameGetInfo);
implement_from!(openxr_sys::InstanceCreateInfo);
implement_from!(openxr_sys::InteractionProfileAnalogThresholdVALVE);
implement_from!(openxr_sys::InteractionProfileSuggestedBinding);
implement_from!(openxr_sys::MarkerSpaceCreateInfoVARJO);
implement_from!(openxr_sys::PassthroughColorMapMonoToMonoFB);
implement_from!(openxr_sys::PassthroughColorMapMonoToRgbaFB);
implement_from!(openxr_sys::PassthroughCreateInfoFB);
implement_from!(openxr_sys::PassthroughKeyboardHandsIntensityFB);
implement_from!(openxr_sys::PassthroughLayerCreateInfoFB);
implement_from!(openxr_sys::PassthroughStyleFB);
implement_from!(openxr_sys::ReferenceSpaceCreateInfo);
implement_from!(openxr_sys::SecondaryViewConfigurationFrameEndInfoMSFT);
implement_from!(openxr_sys::SecondaryViewConfigurationLayerInfoMSFT);
implement_from!(openxr_sys::SecondaryViewConfigurationSessionBeginInfoMSFT);
implement_from!(openxr_sys::SecondaryViewConfigurationSwapchainCreateInfoMSFT);
implement_from!(openxr_sys::SessionActionSetsAttachInfo);
implement_from!(openxr_sys::SessionBeginInfo);
implement_from!(openxr_sys::SessionCreateInfo);
implement_from!(openxr_sys::SessionCreateInfoOverlayEXTX);
implement_from!(openxr_sys::SpatialAnchorCreateInfoMSFT);
implement_from!(openxr_sys::SpatialAnchorFromPersistedAnchorCreateInfoMSFT);
implement_from!(openxr_sys::SpatialAnchorPersistenceInfoMSFT);
implement_from!(openxr_sys::SpatialAnchorSpaceCreateInfoMSFT);
implement_from!(openxr_sys::SpatialGraphNodeSpaceCreateInfoMSFT);
implement_from!(openxr_sys::SwapchainCreateInfo);
implement_from!(openxr_sys::SwapchainImageWaitInfo);
implement_from!(openxr_sys::SystemGetInfo);
implement_from!(openxr_sys::SystemPassthroughPropertiesFB);
implement_from!(openxr_sys::TriangleMeshCreateInfoFB);
implement_from!(openxr_sys::ViewConfigurationViewFovEPIC);
implement_from!(openxr_sys::ViewLocateFoveatedRenderingVARJO);
implement_from!(openxr_sys::ViewLocateInfo);
implement_from!(openxr_sys::VulkanDeviceCreateInfoKHR);
implement_from!(openxr_sys::VulkanGraphicsDeviceGetInfoKHR);
implement_from!(openxr_sys::VulkanInstanceCreateInfoKHR);
implement_from!(openxr_sys::VulkanSwapchainFormatListCreateInfoKHR);
// I somehow missed this one in first version, which means that there likely are
// more missing from the list.
implement_from!(openxr_sys::InteractionProfileState);

impl Iterator for XrStructChain {
    type Item = XrStruct;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let res: *const openxr_sys::BaseInStructure = self.ptr;
        self.ptr = unsafe { (*res).next };
        Some(XrStruct::from(res))
    }
}

#[derive(Clone)]
pub struct SizedArrayValueIterator<T>
where
    T: Copy,
{
    count: u32,
    ptr: *const T,
}

impl<T> SizedArrayValueIterator<T>
where
    T: Copy,
{
    pub(crate) unsafe fn new(count: u32, ptr: *const T) -> Self {
        Self { count, ptr }
    }
}

impl<T: Copy> Iterator for SizedArrayValueIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        let ptr = self.ptr;
        self.ptr = unsafe { ptr.add(1) };
        self.count -= 1;
        Some(unsafe { *ptr })
    }
}

impl<T> std::fmt::Debug for SizedArrayValueIterator<T>
where
    T: Copy + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut f = f.debug_list();
        for item in self.clone() {
            f.entry(&item);
        }
        f.finish()
    }
}

impl<T> XrDebug for SizedArrayValueIterator<T>
where
    T: Copy + XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let mut f = f.debug_list();
        for item in self.clone() {
            f.entry(&XrDebugValue(instance.clone(), &item));
        }
        f.finish()
    }
}
