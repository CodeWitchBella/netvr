using UnityEngine;
using UnityEditor;
using UnityEngine.XR;
using System.Collections.Generic;

namespace Isbl.NetVR.Editor
{

    [CustomPropertyDrawer(typeof(IsblRemoteDevice.SelfPropertyAttribute))]
    public class IsblRemoteDeviceDrawer : PropertyDrawer
    {
        const float LineHeight = 20;
        public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
        {
            var go = property.objectReferenceValue as IsblRemoteDevice;
            return GetDescription(go?.GetComponent<IsblRemoteDevice>()).Count * LineHeight;
        }

        public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
        {
            var go = property.objectReferenceValue as IsblRemoteDevice;
            var device = go?.GetComponent<IsblRemoteDevice>();

            EditorGUI.BeginProperty(position, label, property);
            var y = position.y;
            foreach (var line in GetDescription(device))
            {
                EditorGUI.LabelField(new Rect(position.x, y, position.width, LineHeight), line.Key, line.Value);
                y += LineHeight;
            }
            EditorGUI.EndProperty();
        }

        static List<KeyValuePair<string, string>> GetDescription(IsblRemoteDevice device)
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
                DrawField("Id", device.Id);
            }
            return result;
        }
    }
}
