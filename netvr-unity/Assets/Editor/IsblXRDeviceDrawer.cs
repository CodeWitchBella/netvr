using UnityEngine;
using UnityEditor;
using UnityEngine.XR;

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
            EditorGUI.LabelField(new Rect(position.x, y, position.width, LineHeight), text, text2);
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

            var locations = device.SerializeConfiguration()["locations"];
            foreach (var prop in locations.AsObject())
            {
                var propName = char.ToUpper(prop.Key[0]) + prop.Key[1..];
                if (prop.Value.GetValue<int>() >= 0)
                    DrawField(propName, device.GetType().GetProperty(propName).GetValue(device).ToString());
            }

            var haptics = device.Haptics;
            if (haptics != null)
            {
                const int TextWidth = 75;
                EditorGUI.LabelField(new Rect(position.x, y, TextWidth, 20), "Haptics");
                if (haptics.SupportsImpulse && localDevice != null)
                {
                    if (GUI.Button(new Rect(TextWidth + 10, y, 75, 20), "test"))
                    {
                        localDevice.Device.SendHapticImpulse(0, 0.25f, 0.1f);
                    }
                }
                y += LineHeight;
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
