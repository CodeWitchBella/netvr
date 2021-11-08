using UnityEngine;
using UnityEditor;

[CustomPropertyDrawer(typeof(IsblTrackedPoseDriver.SelfPropertyAttribute))]
public class IsblXRDeviceDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        return LineHeight * 24;
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var go = property.objectReferenceValue as IsblTrackedPoseDriver;
        IsblXRDevice device = go?.GetComponent<IsblTrackedPoseDriver>()?.Device;

        var y = position.y;
        void DrawLine(string text, string text2 = "")
        {
            EditorGUI.LabelField(new Rect(position.x, y, position.width, 20), text, text2);
            y += LineHeight;
        };
        void DrawField<T>(string name, T value, bool native = true)
        {
            DrawLine(name, $"{value}{(native ? "" : " (emulated)")}");
        };


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
            DrawField("PrimaryButton", device.PrimaryButton, device.PrimaryButtonAvailable);
            DrawField("PrimaryTouch", device.PrimaryTouch, device.PrimaryTouchAvailable);
            DrawField("SecondaryButton", device.SecondaryButton, device.SecondaryButtonAvailable);
            DrawField("SecondaryTouch", device.SecondaryTouch, device.SecondaryTouchAvailable);
            DrawField($"TriggerTouch", device.TriggerTouch, device.TriggerTouchAvailable);
            // vive
            DrawField("SystemButton", device.SystemButton, device.SystemButtonAvailable);
        }
        EditorGUI.EndProperty();
    }
}
