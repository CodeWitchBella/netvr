use openxr_sys::pfn;

trait Yup {}
struct Yes {}
impl Yup for Yes {}

macro_rules! implement {
    ($($id: ident$( generate_from=$generate_from: ty)?), *,) => {
        pub enum FnPtr {
            $(
                $id(pfn::$id),
            ) *
        }

        impl FnPtr {
            pub(crate) fn value(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id(_) => concat!("xr", stringify!($id)),
                    ) *
                }
            }

            pub(crate) fn void_fn(&self) -> pfn::VoidFunction {
                match *self {
                    $(
                        Self::$id(ptr) => unsafe { std::mem::transmute(ptr) },
                    ) *
                }
            }
        }

        $(
            $(
                impl From<pfn::$id> for FnPtr where $generate_from: Yup {
                    fn from(ptr: pfn::$id) -> Self {
                        Self::$id(ptr)
                    }
                }
            )?
        ) *
    }
}

implement!(
    //#[cfg(target_os = "android")]
    //SetAndroidApplicationThreadKHR,
    //#[cfg(target_os = "android")]
    //CreateSwapchainAndroidSurfaceKHR,
    DebugUtilsMessengerCallbackEXT generate_from=Yes,
    GetInstanceProcAddr generate_from=Yes,
    EnumerateApiLayerProperties generate_from=Yes,
    EnumerateInstanceExtensionProperties generate_from=Yes,
    CreateInstance generate_from=Yes,
    DestroyInstance generate_from=Yes,
    ResultToString generate_from=Yes,
    StructureTypeToString generate_from=Yes,
    GetInstanceProperties generate_from=Yes,
    GetSystem generate_from=Yes,
    GetSystemProperties generate_from=Yes,
    CreateSession generate_from=Yes,
    DestroySession generate_from=Yes,
    DestroySpace generate_from=Yes,
    EnumerateSwapchainFormats generate_from=Yes,
    CreateSwapchain generate_from=Yes,
    DestroySwapchain generate_from=Yes,
    EnumerateSwapchainImages generate_from=Yes,
    AcquireSwapchainImage generate_from=Yes,
    WaitSwapchainImage generate_from=Yes,
    ReleaseSwapchainImage generate_from=Yes,
    BeginSession generate_from=Yes,
    EndSession,
    RequestExitSession,
    EnumerateReferenceSpaces generate_from=Yes,
    CreateReferenceSpace generate_from=Yes,
    CreateActionSpace generate_from=Yes,
    LocateSpace generate_from=Yes,
    EnumerateViewConfigurations generate_from=Yes,
    EnumerateEnvironmentBlendModes generate_from=Yes,
    GetViewConfigurationProperties generate_from=Yes,
    EnumerateViewConfigurationViews generate_from=Yes,
    BeginFrame generate_from=Yes,
    LocateViews generate_from=Yes,
    EndFrame generate_from=Yes,
    WaitFrame generate_from=Yes,
    ApplyHapticFeedback generate_from=Yes,
    StopHapticFeedback generate_from=Yes,
    PollEvent generate_from=Yes,
    StringToPath generate_from=Yes,
    PathToString generate_from=Yes,
    GetReferenceSpaceBoundsRect generate_from=Yes,
    GetActionStateBoolean generate_from=Yes,
    GetActionStateFloat generate_from=Yes,
    GetActionStateVector2f generate_from=Yes,
    GetActionStatePose generate_from=Yes,
    CreateActionSet generate_from=Yes,
    DestroyActionSet generate_from=Yes,
    CreateAction generate_from=Yes,
    DestroyAction generate_from=Yes,
    SuggestInteractionProfileBindings generate_from=Yes,
    AttachSessionActionSets generate_from=Yes,
    GetCurrentInteractionProfile generate_from=Yes,
    SyncActions generate_from=Yes,
    EnumerateBoundSourcesForAction generate_from=Yes,
    GetInputSourceLocalizedName generate_from=Yes,
    GetVulkanInstanceExtensionsKHR generate_from=Yes,
    GetVulkanDeviceExtensionsKHR,
    GetVulkanGraphicsDeviceKHR generate_from=Yes,
    GetOpenGLGraphicsRequirementsKHR generate_from=Yes,
    GetOpenGLESGraphicsRequirementsKHR generate_from=Yes,
    GetVulkanGraphicsRequirementsKHR generate_from=Yes,
    GetD3D11GraphicsRequirementsKHR generate_from=Yes,
    GetD3D12GraphicsRequirementsKHR generate_from=Yes,
    PerfSettingsSetPerformanceLevelEXT generate_from=Yes,
    ThermalGetTemperatureTrendEXT generate_from=Yes,
    SetDebugUtilsObjectNameEXT generate_from=Yes,
    CreateDebugUtilsMessengerEXT generate_from=Yes,
    DestroyDebugUtilsMessengerEXT generate_from=Yes,
    SubmitDebugUtilsMessageEXT generate_from=Yes,
    SessionBeginDebugUtilsLabelRegionEXT generate_from=Yes,
    SessionEndDebugUtilsLabelRegionEXT,
    SessionInsertDebugUtilsLabelEXT,
    ConvertTimeToWin32PerformanceCounterKHR generate_from=Yes,
    ConvertWin32PerformanceCounterToTimeKHR generate_from=Yes,
    CreateVulkanInstanceKHR generate_from=Yes,
    CreateVulkanDeviceKHR generate_from=Yes,
    GetVulkanGraphicsDevice2KHR generate_from=Yes,
    ConvertTimeToTimespecTimeKHR generate_from=Yes,
    ConvertTimespecTimeToTimeKHR generate_from=Yes,
    GetVisibilityMaskKHR generate_from=Yes,
    CreateSpatialAnchorMSFT generate_from=Yes,
    CreateSpatialAnchorSpaceMSFT generate_from=Yes,
    DestroySpatialAnchorMSFT generate_from=Yes,
    SetInputDeviceActiveEXT generate_from=Yes,
    SetInputDeviceStateBoolEXT,
    SetInputDeviceStateFloatEXT generate_from=Yes,
    SetInputDeviceStateVector2fEXT generate_from=Yes,
    SetInputDeviceLocationEXT generate_from=Yes,
    InitializeLoaderKHR generate_from=Yes,
    CreateSpatialGraphNodeSpaceMSFT generate_from=Yes,
    CreateHandTrackerEXT generate_from=Yes,
    DestroyHandTrackerEXT generate_from=Yes,
    LocateHandJointsEXT generate_from=Yes,
    CreateHandMeshSpaceMSFT generate_from=Yes,
    UpdateHandMeshMSFT generate_from=Yes,
    GetControllerModelKeyMSFT generate_from=Yes,
    LoadControllerModelMSFT generate_from=Yes,
    GetControllerModelPropertiesMSFT generate_from=Yes,
    GetControllerModelStateMSFT generate_from=Yes,
    EnumerateDisplayRefreshRatesFB generate_from=Yes,
    GetDisplayRefreshRateFB generate_from=Yes,
    RequestDisplayRefreshRateFB generate_from=Yes,
    CreateSpatialAnchorFromPerceptionAnchorMSFT generate_from=Yes,
    TryGetPerceptionAnchorFromSpatialAnchorMSFT generate_from=Yes,
    UpdateSwapchainFB generate_from=Yes,
    GetSwapchainStateFB generate_from=Yes,
    EnumerateColorSpacesFB generate_from=Yes,
    SetColorSpaceFB generate_from=Yes,
    CreateFoveationProfileFB generate_from=Yes,
    DestroyFoveationProfileFB generate_from=Yes,
    GetHandMeshFB generate_from=Yes,
    EnumerateRenderModelPathsFB generate_from=Yes,
    GetRenderModelPropertiesFB generate_from=Yes,
    LoadRenderModelFB generate_from=Yes,
    QuerySystemTrackedKeyboardFB generate_from=Yes,
    CreateKeyboardSpaceFB generate_from=Yes,
    SetEnvironmentDepthEstimationVARJO generate_from=Yes,
    EnumerateReprojectionModesMSFT generate_from=Yes,
    GetAudioOutputDeviceGuidOculus generate_from=Yes,
    GetAudioInputDeviceGuidOculus,
    CreateTriangleMeshFB generate_from=Yes,
    DestroyTriangleMeshFB generate_from=Yes,
    TriangleMeshGetVertexBufferFB generate_from=Yes,
    TriangleMeshGetIndexBufferFB generate_from=Yes,
    TriangleMeshBeginUpdateFB,
    TriangleMeshEndUpdateFB generate_from=Yes,
    TriangleMeshBeginVertexBufferUpdateFB generate_from=Yes,
    TriangleMeshEndVertexBufferUpdateFB,
    CreatePassthroughFB generate_from=Yes,
    DestroyPassthroughFB generate_from=Yes,
    PassthroughStartFB,
    PassthroughPauseFB,
    CreatePassthroughLayerFB generate_from=Yes,
    DestroyPassthroughLayerFB generate_from=Yes,
    PassthroughLayerPauseFB,
    PassthroughLayerResumeFB,
    PassthroughLayerSetStyleFB generate_from=Yes,
    CreateGeometryInstanceFB generate_from=Yes,
    DestroyGeometryInstanceFB generate_from=Yes,
    GeometryInstanceSetTransformFB generate_from=Yes,
    PassthroughLayerSetKeyboardHandsIntensityFB generate_from=Yes,
    CreateSpatialAnchorStoreConnectionMSFT generate_from=Yes,
    DestroySpatialAnchorStoreConnectionMSFT generate_from=Yes,
    PersistSpatialAnchorMSFT generate_from=Yes,
    EnumeratePersistedSpatialAnchorNamesMSFT generate_from=Yes,
    CreateSpatialAnchorFromPersistedNameMSFT generate_from=Yes,
    UnpersistSpatialAnchorMSFT generate_from=Yes,
    ClearSpatialAnchorStoreMSFT,
    CreateFacialTrackerHTC generate_from=Yes,
    DestroyFacialTrackerHTC generate_from=Yes,
    GetFacialExpressionsHTC generate_from=Yes,
    EnumerateViveTrackerPathsHTCX generate_from=Yes,
    SetMarkerTrackingVARJO,
    SetMarkerTrackingTimeoutVARJO generate_from=Yes,
    SetMarkerTrackingPredictionVARJO generate_from=Yes,
    GetMarkerSizeVARJO generate_from=Yes,
    CreateMarkerSpaceVARJO generate_from=Yes,
    SetDigitalLensControlALMALENCE generate_from=Yes,
    GetVulkanGraphicsRequirements2KHR,
);
