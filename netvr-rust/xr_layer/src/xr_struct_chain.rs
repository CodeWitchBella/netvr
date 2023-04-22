use crate::{xr_struct::XrStruct, XrDebug};

#[derive(Clone)]
pub struct XrStructChain {
    ptr: *const openxr_sys::BaseInStructure,
}

pub trait UnsafeFrom<T> {
    /// .
    ///
    /// # Safety
    ///
    /// ptr must be valid and point to structure conforming to OpenXR structure
    /// definition (type, next, ...)
    ///
    /// But, it does check for null ptr
    unsafe fn from_ptr(ptr: T) -> Self
    where
        Self: Sized;
}

macro_rules! implement {
    ($method:ident reads $id:ident) => {
        impl UnsafeFrom<*const openxr_sys::$id> for XrStructChain {
            unsafe fn from_ptr(input: *const openxr_sys::$id) -> Self {
                XrStructChain {
                    ptr: unsafe { std::mem::transmute(input) },
                }
            }
        }
        impl UnsafeFrom<*mut openxr_sys::$id> for XrStructChain {
            unsafe fn from_ptr(input: *mut openxr_sys::$id) -> Self {
                XrStructChain {
                    ptr: unsafe { &*(input as *const openxr_sys::BaseInStructure) },
                }
            }
        }

        impl<'a> XrStructChain {
            pub fn $method(&self) -> Result<crate::xr_struct::$id<'a>, openxr_sys::Result> {
                let mut ptr = self.ptr;
                while !ptr.is_null() {
                    let val = unsafe { ptr.read() };
                    if val.ty == openxr_sys::$id::TYPE {
                        return Ok(crate::xr_struct::$id(unsafe {
                            &*std::mem::transmute::<
                                *const openxr_sys::BaseInStructure,
                                *const openxr_sys::$id,
                            >(ptr)
                        }));
                    }
                    ptr = val.next;
                }
                Err(openxr_sys::Result::ERROR_VALIDATION_FAILURE)
            }
        }
    };
}

// Following are missing because they are android-only and I did not want to
// spend time to try and figure out how to integrate them, since I do not
// need them (for now).
//   - AndroidSurfaceSwapchainCreateInfoFB
//   - GraphicsBindingOpenGLESAndroidKHR
//   - InstanceCreateInfoAndroidKHR
//   - LoaderInitInfoAndroidKHR
// Similarly, following are Win32-only.
//   - GraphicsBindingD3D11KHR
//   - GraphicsBindingD3D12KHR
//   - GraphicsBindingOpenGLWin32KHR
//   - HolographicWindowAttachmentMSFT
// And linux-only
//   - GraphicsBindingOpenGLXcbKHR
//
// List of openxr_sys types which can be converted into XrStructChain
implement!(read_action_create_info reads ActionCreateInfo);
implement!(read_action_set_create_info reads ActionSetCreateInfo);
implement!(read_action_space_create_info reads ActionSpaceCreateInfo);
implement!(read_actions_sync_info reads ActionsSyncInfo);
implement!(read_action_state_get_info reads ActionStateGetInfo);
implement!(read_binding_modifications_khr reads BindingModificationsKHR);
implement!(read_bound_sources_for_action_enumerate_info reads BoundSourcesForActionEnumerateInfo);
implement!(read_composition_layer_color_scale_bias_khr reads CompositionLayerColorScaleBiasKHR);
implement!(read_composition_layer_cube_khr reads CompositionLayerCubeKHR);
implement!(read_composition_layer_cylinder_khr reads CompositionLayerCylinderKHR);
implement!(read_composition_layer_depth_info_khr reads CompositionLayerDepthInfoKHR);
implement!(read_composition_layer_depth_test_varjo reads CompositionLayerDepthTestVARJO);
implement!(read_composition_layer_equirect2_khr reads CompositionLayerEquirect2KHR);
implement!(read_composition_layer_equirect_khr reads CompositionLayerEquirectKHR);
implement!(read_composition_layer_passthrough_fb reads CompositionLayerPassthroughFB);
implement!(read_composition_layer_projection reads CompositionLayerProjection);
implement!(read_composition_layer_projection_view reads CompositionLayerProjectionView);
implement!(read_composition_layer_quad reads CompositionLayerQuad);
implement!(read_composition_layer_reprojection_info_msft reads CompositionLayerReprojectionInfoMSFT);
implement!(read_composition_layer_reprojection_plane_override_msft reads CompositionLayerReprojectionPlaneOverrideMSFT);
implement!(read_composition_layer_secure_content_fb reads CompositionLayerSecureContentFB);
implement!(read_composition_layer_space_warp_info_fb reads CompositionLayerSpaceWarpInfoFB);
implement!(read_debug_utils_label_ext reads DebugUtilsLabelEXT);
implement!(read_debug_utils_messenger_callback_data_ext reads DebugUtilsMessengerCallbackDataEXT);
implement!(read_debug_utils_messenger_create_info_ext reads DebugUtilsMessengerCreateInfoEXT);
implement!(read_debug_utils_object_name_info_ext reads DebugUtilsObjectNameInfoEXT);
implement!(read_digital_lens_control_almalence reads DigitalLensControlALMALENCE);
implement!(read_event_data_buffer reads EventDataBuffer);
implement!(read_event_data_display_refresh_rate_changed_fb reads EventDataDisplayRefreshRateChangedFB);
implement!(read_event_data_events_lost reads EventDataEventsLost);
implement!(read_event_data_instance_loss_pending reads EventDataInstanceLossPending);
implement!(read_event_data_interaction_profile_changed reads EventDataInteractionProfileChanged);
implement!(read_event_data_main_session_visibility_changed_extx reads EventDataMainSessionVisibilityChangedEXTX);
implement!(read_event_data_marker_tracking_update_varjo reads EventDataMarkerTrackingUpdateVARJO);
implement!(read_event_data_passthrough_state_changed_fb reads EventDataPassthroughStateChangedFB);
implement!(read_event_data_perf_settings_ext reads EventDataPerfSettingsEXT);
implement!(read_event_data_reference_space_change_pending reads EventDataReferenceSpaceChangePending);
implement!(read_event_data_session_state_changed reads EventDataSessionStateChanged);
implement!(read_event_data_visibility_mask_changed_khr reads EventDataVisibilityMaskChangedKHR);
implement!(read_event_data_vive_tracker_connected_htcx reads EventDataViveTrackerConnectedHTCX);
implement!(read_facial_expressions_htc reads FacialExpressionsHTC);
implement!(read_facial_tracker_create_info_htc reads FacialTrackerCreateInfoHTC);
implement!(read_frame_end_info reads FrameEndInfo);
implement!(read_geometry_instance_create_info_fb reads GeometryInstanceCreateInfoFB);
implement!(read_geometry_instance_transform_fb reads GeometryInstanceTransformFB);
implement!(read_graphics_binding_eglmndx reads GraphicsBindingEGLMNDX);
implement!(read_graphics_binding_open_gl_wayland_khr reads GraphicsBindingOpenGLWaylandKHR);

implement!(read_graphics_binding_open_gl_xlib_khr reads GraphicsBindingOpenGLXlibKHR);
implement!(read_graphics_binding_vulkan_khr reads GraphicsBindingVulkanKHR);
implement!(read_hand_joints_locate_info_ext reads HandJointsLocateInfoEXT);
implement!(read_hand_joints_motion_range_info_ext reads HandJointsMotionRangeInfoEXT);
implement!(read_hand_mesh_space_create_info_msft reads HandMeshSpaceCreateInfoMSFT);
implement!(read_hand_mesh_update_info_msft reads HandMeshUpdateInfoMSFT);
implement!(read_hand_pose_type_info_msft reads HandPoseTypeInfoMSFT);
implement!(read_hand_tracker_create_info_ext reads HandTrackerCreateInfoEXT);
implement!(read_haptic_action_info reads HapticActionInfo);
implement!(read_haptic_vibration reads HapticVibration);
implement!(read_input_source_localized_name_get_info reads InputSourceLocalizedNameGetInfo);
implement!(read_instance_create_info reads InstanceCreateInfo);
implement!(read_interaction_profile_analog_threshold_valve reads InteractionProfileAnalogThresholdVALVE);
implement!(read_interaction_profile_suggested_binding reads InteractionProfileSuggestedBinding);
implement!(read_marker_space_create_info_varjo reads MarkerSpaceCreateInfoVARJO);
implement!(read_passthrough_color_map_mono_to_mono_fb reads PassthroughColorMapMonoToMonoFB);
implement!(read_passthrough_color_map_mono_to_rgba_fb reads PassthroughColorMapMonoToRgbaFB);
implement!(read_passthrough_create_info_fb reads PassthroughCreateInfoFB);
implement!(read_passthrough_keyboard_hands_intensity_fb reads PassthroughKeyboardHandsIntensityFB);
implement!(read_passthrough_layer_create_info_fb reads PassthroughLayerCreateInfoFB);
implement!(read_passthrough_style_fb reads PassthroughStyleFB);
implement!(read_reference_space_create_info reads ReferenceSpaceCreateInfo);
implement!(read_secondary_view_configuration_frame_end_info_msft reads SecondaryViewConfigurationFrameEndInfoMSFT);
implement!(read_secondary_view_configuration_layer_info_msft reads SecondaryViewConfigurationLayerInfoMSFT);
implement!(read_secondary_view_configuration_session_begin_info_msft reads SecondaryViewConfigurationSessionBeginInfoMSFT);
implement!(read_secondary_view_configuration_swapchain_create_info_msft reads SecondaryViewConfigurationSwapchainCreateInfoMSFT);
implement!(read_session_action_sets_attach_info reads SessionActionSetsAttachInfo);
implement!(read_session_begin_info reads SessionBeginInfo);
implement!(read_session_create_info reads SessionCreateInfo);
implement!(read_session_create_info_overlay_extx reads SessionCreateInfoOverlayEXTX);
implement!(read_spatial_anchor_create_info_msft reads SpatialAnchorCreateInfoMSFT);
implement!(read_spatial_anchor_from_persisted_anchor_create_info_msft reads SpatialAnchorFromPersistedAnchorCreateInfoMSFT);
implement!(read_spatial_anchor_persistence_info_msft reads SpatialAnchorPersistenceInfoMSFT);
implement!(read_spatial_anchor_space_create_info_msft reads SpatialAnchorSpaceCreateInfoMSFT);
implement!(read_spatial_graph_node_space_create_info_msft reads SpatialGraphNodeSpaceCreateInfoMSFT);
implement!(read_swapchain_create_info reads SwapchainCreateInfo);
implement!(read_swapchain_image_wait_info reads SwapchainImageWaitInfo);
implement!(read_system_get_info reads SystemGetInfo);
implement!(read_system_passthrough_properties_fb reads SystemPassthroughPropertiesFB);
implement!(read_triangle_mesh_create_info_fb reads TriangleMeshCreateInfoFB);
implement!(read_view_configuration_view_fov_epic reads ViewConfigurationViewFovEPIC);
implement!(read_view_locate_foveated_rendering_varjo reads ViewLocateFoveatedRenderingVARJO);
implement!(read_view_locate_info reads ViewLocateInfo);
implement!(read_vulkan_device_create_info_khr reads VulkanDeviceCreateInfoKHR);
implement!(read_vulkan_graphics_device_get_info_khr reads VulkanGraphicsDeviceGetInfoKHR);
implement!(read_vulkan_instance_create_info_khr reads VulkanInstanceCreateInfoKHR);
implement!(read_vulkan_swapchain_format_list_create_info_khr reads VulkanSwapchainFormatListCreateInfoKHR);
// I somehow missed this one in first version, which means that there likely are
// more missing from the list.
implement!(read_interaction_profile_state reads InteractionProfileState);
implement!(read_action_state_boolean reads ActionStateBoolean);

impl XrDebug for XrStructChain {
    fn xr_fmt(&self, f: &mut std::fmt::Formatter, instance: &openxr::Instance) -> std::fmt::Result {
        let mut f = f.debug_list();
        let mut ptr = self.ptr;
        while !ptr.is_null() {
            let val = unsafe { ptr.read() };
            f.entry(&XrStruct::from(ptr).as_debug(instance));
            ptr = val.next;
        }

        f.finish()
    }
}
