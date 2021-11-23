using UnityEngine;
using UnityEditor;
using UnityEngine.XR;

[CustomPropertyDrawer(typeof(IsblTrackedPoseDriver.SelfPropertyAttribute))]
public class IsblXRDeviceDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        return LineHeight * 27;
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var go = property.objectReferenceValue as IsblTrackedPoseDriver;
        IsblStaticXRDevice device = go?.GetComponent<IsblTrackedPoseDriver>()?.NetDevice;
        var localDeviceComponent = go?.GetComponent<IsblXRDeviceComponent>();
        var localDevice = localDeviceComponent?.Device;

        var y = position.y;
        void DrawLine(string text, string text2 = "")
        {
            EditorGUI.LabelField(new Rect(position.x, y, position.width, 20), text, text2);
            y += LineHeight;
        }
        void DrawField<T>(string name, T value, bool native = true)
        {
            DrawLine(name, $"{value}{(native ? "" : " (emulated)")}");
        }

        EditorGUI.BeginProperty(position, label, property);
        if (go == null)
        {
            DrawLine("Could not get game object.");
        }
        else if (device == null)
        {
            DrawLine("No device attached.");
        }
        else
        {
            DrawField("Device", localDeviceComponent == null ? "Networked" : localDevice == null ? "Local (disconnected)" : "Local");
            DrawField("Name", device.Name);
            DrawField("Characteristics", SerializeCharacteristics(device.Characteristics));
            DrawField("DeviceAngularVelocity", device.DeviceAngularVelocity, device.DeviceAngularVelocityAvailable);
            DrawField("DevicePosition", device.DevicePosition, device.DevicePositionAvailable);
            DrawField("DeviceRotation", device.DeviceRotation, device.DeviceRotationAvailable);
            DrawField("DeviceVelocity", device.DeviceVelocity, device.DeviceVelocityAvailable);
            DrawField("Grip", device.Grip, device.GripAvailable);
            DrawField("GripButton", device.GripButton, device.GripButtonAvailable);
            DrawField("IsTracked", device.IsTracked, device.IsTrackedAvailable);
            DrawField("MenuButton", device.MenuButton, device.MenuButtonAvailable);
            DrawField("PointerAngularVelocity", device.PointerAngularVelocity, device.PointerAngularVelocityAvailable);
            DrawField("PointerPosition", device.PointerPosition, device.PointerPositionAvailable);
            DrawField("PointerRotation", device.PointerRotation, device.PointerRotationAvailable);
            DrawField("PointerVelocity", device.PointerVelocity, device.PointerVelocityAvailable);
            DrawField("Primary2DAxis", device.Primary2DAxis, device.Primary2DAxisAvailable);
            DrawField("Primary2DAxisClick", device.Primary2DAxisClick, device.Primary2DAxisClickAvailable);
            DrawField("Primary2DAxisTouch", device.Primary2DAxisTouch, device.Primary2DAxisTouchAvailable);
            DrawField("TrackingState", device.TrackingState, device.TrackingStateAvailable);
            DrawField("Trigger", device.Trigger, device.TriggerAvailable);
            DrawField("TriggerButton", device.TriggerButton, device.TriggerButtonAvailable);
            // oculus touch
            DrawField("PrimaryButton", device.PrimaryButton, device.PrimaryButtonAvailable);
            DrawField("PrimaryTouch", device.PrimaryTouch, device.PrimaryTouchAvailable);
            DrawField("SecondaryButton", device.SecondaryButton, device.SecondaryButtonAvailable);
            DrawField("SecondaryTouch", device.SecondaryTouch, device.SecondaryTouchAvailable);
            DrawField("TriggerTouch", device.TriggerTouch, device.TriggerTouchAvailable);
            // vive
            DrawField("SystemButton", device.SystemButton, device.SystemButtonAvailable);
        }
        EditorGUI.EndProperty();
    }

    static string SerializeCharacteristics(InputDeviceCharacteristics c)
    {
        if (c == 0) return "none";
        var res = "";
        void Check(InputDeviceCharacteristics reference)
        {
            if ((c & reference) != 0)
            {
                if (res.Length > 0) res += ", ";
                res += reference.ToString();
            }
        }
        Check(InputDeviceCharacteristics.HeadMounted);
        Check(InputDeviceCharacteristics.Camera);
        Check(InputDeviceCharacteristics.HeldInHand);
        Check(InputDeviceCharacteristics.HandTracking);
        Check(InputDeviceCharacteristics.EyeTracking);
        Check(InputDeviceCharacteristics.TrackedDevice);
        Check(InputDeviceCharacteristics.Controller);
        Check(InputDeviceCharacteristics.TrackingReference);
        Check(InputDeviceCharacteristics.Left);
        Check(InputDeviceCharacteristics.Right);
        Check(InputDeviceCharacteristics.Simulated6DOF);
        return res;
    }
}
