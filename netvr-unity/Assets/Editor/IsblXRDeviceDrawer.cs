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
            DrawField("DeviceAngularVelocity", device.DeviceAngularVelocity);
            DrawField("DevicePosition", device.DevicePosition);
            DrawField("DeviceRotation", device.DeviceRotation);
            DrawField("DeviceVelocity", device.DeviceVelocity);
            DrawField("Grip", device.Grip);
            DrawField("GripButton", device.GripButton);
            DrawField("IsTracked", device.IsTracked);
            DrawField("MenuButton", device.MenuButton);
            DrawField("PointerAngularVelocity", device.PointerAngularVelocity);
            DrawField("PointerPosition", device.PointerPosition);
            DrawField("PointerRotation", device.PointerRotation);
            DrawField("PointerVelocity", device.PointerVelocity);
            DrawField("Primary2DAxis", device.Primary2DAxis);
            DrawField("Primary2DAxisClick", device.Primary2DAxisClick);
            DrawField("Primary2DAxisTouch", device.Primary2DAxisTouch);
            DrawField("TrackingState", device.TrackingState);
            DrawField("Trigger", device.Trigger);
            DrawField("TriggerButton", device.TriggerButton);
            // oculus touch
            DrawField("PrimaryButton", device.PrimaryButton, localDevice?.PrimaryButtonAvailable ?? true);
            DrawField("PrimaryTouch", device.PrimaryTouch, localDevice?.PrimaryTouchAvailable ?? true);
            DrawField("SecondaryButton", device.SecondaryButton, localDevice?.SecondaryButtonAvailable ?? true);
            DrawField("SecondaryTouch", device.SecondaryTouch, localDevice?.SecondaryTouchAvailable ?? true);
            DrawField("TriggerTouch", device.TriggerTouch, localDevice?.TriggerTouchAvailable ?? true);
            // vive
            DrawField("SystemButton", device.SystemButton, localDevice?.SystemButtonAvailable ?? true);
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
