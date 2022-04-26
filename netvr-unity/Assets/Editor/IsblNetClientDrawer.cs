using UnityEngine;
using UnityEditor;
using System.Collections.Generic;

[CustomPropertyDrawer(typeof(IsblNetRemoteClient.SelfPropertyAttribute))]
public class IsblNetClientDrawer : PropertyDrawer
{
    const float LineHeight = 20;

    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        return LineHeight * GetInfo(property).Count;
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var y = position.y;
        EditorGUI.BeginProperty(position, label, property);
        foreach (var pair in GetInfo(property))
        {
            EditorGUI.LabelField(new Rect(position.x, y, position.width, 20), pair.Key, pair.Value);
            y += LineHeight;
        }
        EditorGUI.EndProperty();
    }

    List<KeyValuePair<string, string>> GetInfo(SerializedProperty property)
    {
        var clientComponent = (IsblNetRemoteClient)property.serializedObject.targetObject;
        List<KeyValuePair<string, string>> res = new();
        if (clientComponent == null)
        {
            res.Add(new("Couldn't retrieve IsblNetRemoteClient", ""));
            return res;
        }
        res.Add(new("Id", $"{clientComponent.Id}"));
        var clients = IsblNet.Instance?.ServerState?.Clients;
        if (clients == null || !clients.TryGetValue(clientComponent.Id, out var client))
        {
            res.Add(new("Couldn't retrieve Isbl.NetServerState.Client", ""));
            return res;
        }

        res.Add(new("Connected", client.Connected + ""));
        if (client.Devices == null)
        {
            res.Add(new("Devices", "null"));
            res.Add(new("-- info --", "likely is web console (no devices)"));
            return res;
        }
        res.Add(new("Devices.Count", client.Devices.Count + ""));
        res.Add(new("Calibration", ""));
        res.Add(new("  Rotate", OutputVec3(client.Calibration.Rotate.eulerAngles)));
        res.Add(new("  Scale", OutputVec3(client.Calibration.Scale)));
        res.Add(new("  Translate", OutputVec3(client.Calibration.Translate)));

        return res;
    }

    string OutputVec3(Vector3 v)
    {
        return $"{{ x: {v.x}, y: {v.y}, z: {v.z} }}";
    }
}
