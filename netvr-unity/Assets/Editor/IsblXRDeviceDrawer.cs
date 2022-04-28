using UnityEngine;
using UnityEditor;
using UnityEngine.XR;
using System.Collections.Generic;

[CustomPropertyDrawer(typeof(IsblTrackedPoseDriver.SelfPropertyAttribute))]
public class IsblXRDeviceDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        var go = property.objectReferenceValue as IsblTrackedPoseDriver;
        return GetDescription(go?.GetComponent<IsblTrackedPoseDriver>()?.NetDevice).Count * LineHeight;
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var go = property.objectReferenceValue as IsblTrackedPoseDriver;
        var driver = go?.GetComponent<IsblTrackedPoseDriver>();
        IsblStaticXRDevice device = driver?.NetDevice;
        var localDevice = driver?.LocalDevice;

        EditorGUI.BeginProperty(position, label, property);
        var y = position.y;
        foreach (var line in GetDescription(device))
        {
            if (line.Key == "Haptics")
            {
                const int TextWidth = 75;
                if (device?.Haptics?.SupportsImpulse == true && localDevice != null)
                {
                    if (GUI.Button(new Rect(TextWidth + 10, y, 75, 20), "test"))
                    {
                        localDevice.Device.SendHapticImpulse(0, 0.25f, 0.1f);
                    }
                }
            }

            EditorGUI.LabelField(new Rect(position.x, y, position.width, LineHeight), line.Key, line.Value);
            y += LineHeight;
        }
        EditorGUI.EndProperty();
    }

    static List<KeyValuePair<string, string>> GetDescription(IsblStaticXRDevice device)
    {

        List<KeyValuePair<string, string>> result = new();
        void DrawLine(string text, string text2 = "")
        {
            result.Add(new(text, text2));
        }
        void DrawField<T>(string name, T value, bool native = true)
        {
            DrawLine(name, $"{value}{(native ? "" : " (emulated)")}");
        }

        if (device == null)
        {
            DrawLine("No device attached.");
        }
        else
        {
            DrawField("Device", !device.IsLocal ? "Networked" : device.TrackingState == 0 ? "Local (disconnected)" : "Local");
            DrawField("LocallyUniqueId", device.LocallyUniqueId);
            DrawField("Name", device.Name);
            DrawField("SerialNumber", device.SerialNumber);
            DrawField("Characteristics", SerializeCharacteristics(device.Characteristics));
            DrawField("HasData", device.HasData);

            var locations = device.SerializeConfiguration().GetProperty("locations");
            foreach (var prop in locations.EnumerateObject())
            {
                var propName = char.ToUpper(prop.Name[0]) + prop.Name[1..];
                if (prop.Value.GetInt32() >= 0)
                    DrawField(propName, device.GetType().GetProperty(propName).GetValue(device).ToString());
            }

            var haptics = device.Haptics;
            if (haptics != null)
            {
                DrawLine("Haptics");
                DrawLine("    NumChannels", haptics.NumChannels.ToString());
                DrawLine("    SupportsImpulse", haptics.SupportsImpulse.ToString());
                DrawLine("    SupportsBuffer", haptics.SupportsBuffer.ToString());
                DrawLine("    BufferFrequencyHz", haptics.BufferFrequencyHz.ToString());
                DrawLine("    BufferMaxSize", haptics.BufferMaxSize.ToString());
                DrawLine("    BufferOptimalSize", haptics.BufferOptimalSize.ToString());
            }
            else
            {
                DrawLine("Haptics", "none");
            }
        }
        return result;
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
