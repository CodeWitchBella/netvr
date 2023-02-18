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

macro_rules! implement_readers {
    ($( $method: ident reads $id: ident), *,) => {
        $(
            #[repr(transparent)]
            pub struct $id<'a>(&'a openxr_sys::$id);

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

        impl XrStruct {
            $(
                /// Checks if structure is of given type and returns it if yes.
                /// Returns None otherwise.
                #[allow(dead_code)]
                pub fn $method<'a>(&'a self) -> Option<$id<'a>> {
                    if self.data.is_null() { return None; }
                    if self.ty != openxr_sys::$id::TYPE { return None; }
                    Some($id(unsafe {
                        &*std::mem::transmute::<
                            *const openxr_sys::BaseInStructure,
                            *const openxr_sys::$id,
                        >(self.data)
                    }))
                }
            )*
        }

        impl XrDebug for XrStruct {
            fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
                match self.ty {
                    $(
                        openxr_sys::$id::TYPE => {
                            match self.$method() {
                                Some(v) => v.xr_fmt(f, instance),
                                None => f.debug_struct("None").finish(),
                            }
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

implement_readers!(
    // Following are missing because they are android-only and I did not want to
    // spend time to try and figure out how to integrate them, since I do not
    // need them (for now).
    //   - read_android_surface_swapchain_create_info_fb reads AndroidSurfaceSwapchainCreateInfoFB,
    //   - read_graphics_binding_open_gles_android_khr reads GraphicsBindingOpenGLESAndroidKHR,
    //   - read_instance_create_info_android_khr reads InstanceCreateInfoAndroidKHR,
    //   - read_loader_init_info_android_khr reads LoaderInitInfoAndroidKHR,
    read_action_create_info reads ActionCreateInfo,
    read_action_set_create_info reads ActionSetCreateInfo,
    read_action_space_create_info reads ActionSpaceCreateInfo,
    read_action_state_get_info reads ActionStateGetInfo,
    read_actions_sync_info reads ActionsSyncInfo,
    read_binding_modifications_khr reads BindingModificationsKHR,
    read_bound_sources_for_action_enumerate_info reads BoundSourcesForActionEnumerateInfo,
    read_composition_layer_color_scale_bias_khr reads CompositionLayerColorScaleBiasKHR,
    read_composition_layer_cube_khr reads CompositionLayerCubeKHR,
    read_composition_layer_cylinder_khr reads CompositionLayerCylinderKHR,
    read_composition_layer_depth_info_khr reads CompositionLayerDepthInfoKHR,
    read_composition_layer_depth_test_varjo reads CompositionLayerDepthTestVARJO,
    read_composition_layer_equirect_khr reads CompositionLayerEquirectKHR,
    read_composition_layer_equirect2_khr reads CompositionLayerEquirect2KHR,
    read_composition_layer_passthrough_fb reads CompositionLayerPassthroughFB,
    read_composition_layer_projection reads CompositionLayerProjection,
    read_composition_layer_projection_view reads CompositionLayerProjectionView,
    read_composition_layer_quad reads CompositionLayerQuad,
    read_composition_layer_reprojection_info_msft reads CompositionLayerReprojectionInfoMSFT,
    read_composition_layer_reprojection_plane_override_msft reads CompositionLayerReprojectionPlaneOverrideMSFT,
    read_composition_layer_secure_content_fb reads CompositionLayerSecureContentFB,
    read_composition_layer_space_warp_info_fb reads CompositionLayerSpaceWarpInfoFB,
    read_debug_utils_label_ext reads DebugUtilsLabelEXT,
    read_debug_utils_messenger_callback_data_ext reads DebugUtilsMessengerCallbackDataEXT,
    read_debug_utils_messenger_create_info_ext reads DebugUtilsMessengerCreateInfoEXT,
    read_debug_utils_object_name_info_ext reads DebugUtilsObjectNameInfoEXT,
    read_digital_lens_control_almalence reads DigitalLensControlALMALENCE,
    read_event_data_buffer reads EventDataBuffer,
    read_event_data_display_refresh_rate_changed_fb reads EventDataDisplayRefreshRateChangedFB,
    read_event_data_events_lost reads EventDataEventsLost,
    read_event_data_instance_loss_pending reads EventDataInstanceLossPending,
    read_event_data_interaction_profile_changed reads EventDataInteractionProfileChanged,
    read_event_data_main_session_visibility_changed_extx reads EventDataMainSessionVisibilityChangedEXTX,
    read_event_data_marker_tracking_update_varjo reads EventDataMarkerTrackingUpdateVARJO,
    read_event_data_passthrough_state_changed_fb reads EventDataPassthroughStateChangedFB,
    read_event_data_perf_settings_ext reads EventDataPerfSettingsEXT,
    read_event_data_reference_space_change_pending reads EventDataReferenceSpaceChangePending,
    read_event_data_session_state_changed reads EventDataSessionStateChanged,
    read_event_data_visibility_mask_changed_khr reads EventDataVisibilityMaskChangedKHR,
    read_event_data_vive_tracker_connected_htcx reads EventDataViveTrackerConnectedHTCX,
    read_facial_expressions_htc reads FacialExpressionsHTC,
    read_facial_tracker_create_info_htc reads FacialTrackerCreateInfoHTC,
    read_frame_end_info reads FrameEndInfo,
    read_geometry_instance_create_info_fb reads GeometryInstanceCreateInfoFB,
    read_geometry_instance_transform_fb reads GeometryInstanceTransformFB,
    //read_graphics_binding_d3_d11_khr reads GraphicsBindingD3D11KHR,
    //read_graphics_binding_d3_d12_khr reads GraphicsBindingD3D12KHR,
    read_graphics_binding_eglmndx reads GraphicsBindingEGLMNDX,
    read_graphics_binding_open_gl_wayland_khr reads GraphicsBindingOpenGLWaylandKHR,
    //read_graphics_binding_open_gl_win32_khr reads GraphicsBindingOpenGLWin32KHR,
    //read_graphics_binding_open_gl_xcb_khr reads GraphicsBindingOpenGLXcbKHR,
    read_graphics_binding_open_gl_xlib_khr reads GraphicsBindingOpenGLXlibKHR,
    read_graphics_binding_vulkan_khr reads GraphicsBindingVulkanKHR,
    read_hand_joints_locate_info_ext reads HandJointsLocateInfoEXT,
    read_hand_joints_motion_range_info_ext reads HandJointsMotionRangeInfoEXT,
    read_hand_mesh_space_create_info_msft reads HandMeshSpaceCreateInfoMSFT,
    read_hand_mesh_update_info_msft reads HandMeshUpdateInfoMSFT,
    read_hand_pose_type_info_msft reads HandPoseTypeInfoMSFT,
    read_hand_tracker_create_info_ext reads HandTrackerCreateInfoEXT,
    read_haptic_action_info reads HapticActionInfo,
    read_haptic_vibration reads HapticVibration,
    //read_holographic_window_attachment_msft reads HolographicWindowAttachmentMSFT,
    read_input_source_localized_name_get_info reads InputSourceLocalizedNameGetInfo,
    read_instance_create_info reads InstanceCreateInfo,
    read_interaction_profile_analog_threshold_valve reads InteractionProfileAnalogThresholdVALVE,
    read_interaction_profile_suggested_binding reads InteractionProfileSuggestedBinding,
    read_marker_space_create_info_varjo reads MarkerSpaceCreateInfoVARJO,
    read_passthrough_color_map_mono_to_mono_fb reads PassthroughColorMapMonoToMonoFB,
    read_passthrough_color_map_mono_to_rgba_fb reads PassthroughColorMapMonoToRgbaFB,
    read_passthrough_create_info_fb reads PassthroughCreateInfoFB,
    read_passthrough_keyboard_hands_intensity_fb reads PassthroughKeyboardHandsIntensityFB,
    read_passthrough_layer_create_info_fb reads PassthroughLayerCreateInfoFB,
    read_passthrough_style_fb reads PassthroughStyleFB,
    read_reference_space_create_info reads ReferenceSpaceCreateInfo,
    read_secondary_view_configuration_frame_end_info_msft reads SecondaryViewConfigurationFrameEndInfoMSFT,
    read_secondary_view_configuration_layer_info_msft reads SecondaryViewConfigurationLayerInfoMSFT,
    read_secondary_view_configuration_session_begin_info_msft reads SecondaryViewConfigurationSessionBeginInfoMSFT,
    read_secondary_view_configuration_swapchain_create_info_msft reads SecondaryViewConfigurationSwapchainCreateInfoMSFT,
    read_session_action_sets_attach_info reads SessionActionSetsAttachInfo,
    read_session_begin_info reads SessionBeginInfo,
    read_session_create_info reads SessionCreateInfo,
    read_session_create_info_overlay_extx reads SessionCreateInfoOverlayEXTX,
    read_spatial_anchor_create_info_msft reads SpatialAnchorCreateInfoMSFT,
    read_spatial_anchor_from_persisted_anchor_create_info_msft reads SpatialAnchorFromPersistedAnchorCreateInfoMSFT,
    read_spatial_anchor_persistence_info_msft reads SpatialAnchorPersistenceInfoMSFT,
    read_spatial_anchor_space_create_info_msft reads SpatialAnchorSpaceCreateInfoMSFT,
    read_spatial_graph_node_space_create_info_msft reads SpatialGraphNodeSpaceCreateInfoMSFT,
    read_swapchain_create_info reads SwapchainCreateInfo,
    read_swapchain_image_wait_info reads SwapchainImageWaitInfo,
    read_system_get_info reads SystemGetInfo,
    read_system_passthrough_properties_fb reads SystemPassthroughPropertiesFB,
    read_triangle_mesh_create_info_fb reads TriangleMeshCreateInfoFB,
    read_view_configuration_view_fov_epic reads ViewConfigurationViewFovEPIC,
    read_view_locate_foveated_rendering_varjo reads ViewLocateFoveatedRenderingVARJO,
    read_view_locate_info reads ViewLocateInfo,
    read_vulkan_device_create_info_khr reads VulkanDeviceCreateInfoKHR,
    read_vulkan_graphics_device_get_info_khr reads VulkanGraphicsDeviceGetInfoKHR,
    read_vulkan_instance_create_info_khr reads VulkanInstanceCreateInfoKHR,
    read_vulkan_swapchain_format_list_create_info_khr reads VulkanSwapchainFormatListCreateInfoKHR,
    // I somehow missed this one in first version, which means that there likely
    // are more missing from the list.
    read_interaction_profile_state reads InteractionProfileState,
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
