using UnityEngine;
using UnityEditor;
using UnityEngine.XR;
using Newtonsoft.Json.Linq;

[CustomPropertyDrawer(typeof(IsblTrackedPoseDriver.SelfPropertyAttribute))]
public class IsblXRDeviceDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        return LineHeight * 31;
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var go = property.objectReferenceValue as IsblTrackedPoseDriver;
        var driver = go?.GetComponent<IsblTrackedPoseDriver>();
        IsblStaticXRDevice device = driver?.NetDevice;
        var localDevice = driver?.LocalDevice;

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
            DrawField("Device", !device.IsLocal ? "Networked" : device.TrackingState == 0 ? "Local (disconnected)" : "Local");
            DrawField("LocallyUniqueId", device.LocallyUniqueId);
            DrawField("Name", device.Name);
            DrawField("Characteristics", SerializeCharacteristics(device.Characteristics));
            DrawField("HasData", device.HasData);

            var locations = device.SerializeConfiguration().Value<JObject>("locations");
            foreach (var prop in locations.Properties())
            {
                var propName = char.ToUpper(prop.Name[0]) + prop.Name[1..];
                if (prop.Value.Value<int>() >= 0)
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
