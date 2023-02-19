using System.Collections.Generic;
using UnityEngine.Scripting;
using UnityEngine.XR.OpenXR.Input;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Layouts;
using UnityEngine.InputSystem.Controls;
using UnityEngine.InputSystem.XR;
using UnityEngine.XR.OpenXR.Features;
using UnityEngine.XR;

#if UNITY_EDITOR
using UnityEditor;
#endif

using PoseControl = UnityEngine.InputSystem.XR.PoseControl;

namespace Isbl.NetVR
{
    /// <summary>
    /// This <see cref="OpenXRInteractionFeature"/> enables the use of Remote Headset Controllers interaction profiles in OpenXR.
    /// </summary>
#if UNITY_EDITOR
    [UnityEditor.XR.OpenXR.Features.OpenXRFeature(UiName = "NetVR Remote Headset Controller Profile",
        BuildTargetGroups = new[] { BuildTargetGroup.Standalone, BuildTargetGroup.WSA, BuildTargetGroup.Android },
        Company = "Isbl",
        Desc = "Allows showing remote headset as a \"controller\".",
        //DocumentationLink = Constants.k_DocumentationManualURL + "features/valveindexcontrollerprofile.html",
        OpenxrExtensionStrings = "",
        Version = "0.0.1",
        Category =  UnityEditor.XR.OpenXR.Features.FeatureCategory.Interaction,
        FeatureId = featureId)]
#endif
    public class NetVRRemoteHeadsetControllerProfile : OpenXRInteractionFeature
    {
        /// <summary>
        /// The feature id string. This is used to give the feature a well known id for reference.
        /// </summary>
        public const string featureId = "cz.isbl.netvr.feature.input.headset";

        /// <summary>
        /// An Input System device for NetVR Remote Headset.
        /// </summary>
        [Preserve, InputControlLayout(displayName = "Remote Headset Controller (OpenXR)", commonUsages = new string[] { })]
        public class NetVRRemoteHeadsetController : XRController
        {
            /// <summary>
            /// A <see cref="PoseControl"/> that represents the grip pose OpenXR binding.
            /// </summary>
            [Preserve, InputControl(offset = 0, aliases = new[] { "device", "gripPose" }, usage = "Device")]
            public PoseControl devicePose { get; private set; }

            /// <summary>
            /// A <see cref="PoseControl"/> that represents the Valve Index Controller Profile pointer OpenXR binding.
            /// </summary>
            [Preserve, InputControl(offset = 0, alias = "aimPose", usage = "Pointer")]
            public PoseControl pointer { get; private set; }
            /*
                        /// <summary>
                        /// A [ButtonControl](xref:UnityEngine.InputSystem.Controls.ButtonControl) required for backwards compatibility with the XRSDK layouts. This represents the overall tracking state of the device. This value is equivalent to mapping devicePose/isTracked.
                        /// </summary>
                        [Preserve, InputControl(offset = 53)]
                        new public ButtonControl isTracked { get; private set; }

                        /// <summary>
                        /// A [IntegerControl](xref:UnityEngine.InputSystem.Controls.IntegerControl) required for backwards compatibility with the XRSDK layouts. This represents the bit flag set indicating what data is valid. This value is equivalent to mapping devicePose/trackingState.
                        /// </summary>
                        [Preserve, InputControl(offset = 56)]
                        new public IntegerControl trackingState { get; private set; }

                        /// <summary>
                        /// A [Vector3Control](xref:UnityEngine.InputSystem.Controls.Vector3Control) required for backwards compatibility with the XRSDK layouts. This is the device position, or grip position. This value is equivalent to mapping devicePose/position.
                        /// </summary>
                        [Preserve, InputControl(offset = 60, alias = "gripPosition")]
                        new public Vector3Control devicePosition { get; private set; }

                        /// <summary>
                        /// A [QuaternionControl](xref:UnityEngine.InputSystem.Controls.QuaternionControl) required for backwards compatibility with the XRSDK layouts. This is the device orientation, or grip orientation. This value is equivalent to mapping devicePose/rotation.
                        /// </summary>
                        [Preserve, InputControl(offset = 72, alias = "gripOrientation")]
                        new public QuaternionControl deviceRotation { get; private set; }

                        /// <summary>
                        /// A [Vector3Control](xref:UnityEngine.InputSystem.Controls.Vector3Control) required for backwards compatibility with the XRSDK layouts. This is the pointer position. This value is equivalent to mapping pointerPose/position.
                        /// </summary>
                        [Preserve, InputControl(offset = 120)]
                        public Vector3Control pointerPosition { get; private set; }

                        /// <summary>
                        /// A [QuaternionControl](xref:UnityEngine.InputSystem.Controls.QuaternionControl) required for backwards compatibility with the XRSDK layouts. This is the pointer rotation. This value is equivalent to mapping pointerPose/rotation.
                        /// </summary>
                        [Preserve, InputControl(offset = 132, alias = "pointerOrientation")]
                        public QuaternionControl pointerRotation { get; private set; }
            */

            /// <inheritdoc  cref="OpenXRDevice"/>
            protected override void FinishSetup()
            {
                base.FinishSetup();

                devicePose = GetChildControl<PoseControl>("devicePose");
                pointer = GetChildControl<PoseControl>("pointer");
                /*
                isTracked = GetChildControl<ButtonControl>("isTracked");
                trackingState = GetChildControl<IntegerControl>("trackingState");
                devicePosition = GetChildControl<Vector3Control>("devicePosition");
                deviceRotation = GetChildControl<QuaternionControl>("deviceRotation");
                pointerPosition = GetChildControl<Vector3Control>("pointerPosition");
                pointerRotation = GetChildControl<QuaternionControl>("pointerRotation");
                */
            }
        }

        /// <summary>
        /// The interaction profile string used to reference the Remote Headset
        /// </summary>
        public const string profile = "/interaction_profiles/isbl/remote_headset";

        /// <summary>
        /// Constant for a pose interaction binding '.../input/grip/pose' OpenXR Input Binding.
        /// </summary>
        public const string grip = "/input/grip/pose";
        /// <summary>
        /// Constant for a pose interaction binding '.../input/aim/pose' OpenXR Input Binding.
        /// </summary>
        public const string aim = "/input/aim/pose";


        private const string kDeviceLocalizedName = "NetVR Remote Headset";

        /// <summary>
        /// Registers the <see cref="NetVRRemoteHeadsetController"/> layout with the Input System.
        /// </summary>
        protected override void RegisterDeviceLayout()
        {
            InputSystem.RegisterLayout(typeof(NetVRRemoteHeadsetController),
                        matches: new InputDeviceMatcher()
                        .WithInterface(XRUtilities.InterfaceMatchAnyVersion)
                        .WithProduct(kDeviceLocalizedName));
        }

        /// <summary>
        /// Removes the <see cref="NetVRRemoteHeadsetController"/> layout from the Input System.
        /// </summary>
        protected override void UnregisterDeviceLayout()
        {
            InputSystem.RemoveLayout(nameof(NetVRRemoteHeadsetController));
        }

        /// <inheritdoc/>
        protected override void RegisterActionMapsWithRuntime()
        {
            ActionMapConfig actionMap = new ActionMapConfig()
            {
                name = "netvrremoteheadsetcontroller",
                localizedName = kDeviceLocalizedName,
                desiredInteractionProfile = profile,
                manufacturer = "Isbl",
                serialNumber = "",
                deviceInfos = new List<DeviceConfig>()
                {
                    new DeviceConfig()
                    {
                        // TODO
                        characteristics = (InputDeviceCharacteristics)(InputDeviceCharacteristics.HeldInHand | InputDeviceCharacteristics.TrackedDevice | InputDeviceCharacteristics.Controller | InputDeviceCharacteristics.Left),
                        // TODO
                        userPath = UserPaths.leftHand
                    },
                },
                actions = new List<ActionConfig>()
                {
                    // Device Pose
                    new ActionConfig()
                    {
                        name = "devicePose",
                        localizedName = "Device Pose",
                        type = ActionType.Pose,
                        usages = new List<string>()
                        {
                            "Device"
                        },
                        bindings = new List<ActionBinding>()
                        {
                            new ActionBinding()
                            {
                                interactionPath = grip,
                                interactionProfileName = profile,
                            }
                        }
                    },
                    // Pointer Pose
                    new ActionConfig()
                    {
                        name = "pointer",
                        localizedName = "Pointer Pose",
                        type = ActionType.Pose,
                        usages = new List<string>()
                        {
                            "Pointer"
                        },
                        bindings = new List<ActionBinding>()
                        {
                            new ActionBinding()
                            {
                                interactionPath = aim,
                                interactionProfileName = profile,
                            }
                        }
                    },
                }
            };

            AddActionMap(actionMap);
        }
    }
}
