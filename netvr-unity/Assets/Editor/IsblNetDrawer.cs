using UnityEngine;
using UnityEditor;
using System.Text.Json;
using System.Linq;
using System.Collections.Generic;
using System.Text.RegularExpressions;

[CustomPropertyDrawer(typeof(IsblNetComponent.SelfPropertyAttribute))]
public class IsblNetDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    string _url;

    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        var net = IsblNetComponent.InstanceExists ? IsblNetComponent.Instance : null;
        return LineHeight * (24 + SerializeIsblNet(net).Count);
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var net = IsblNetComponent.InstanceExists ? IsblNetComponent.Instance : null;

        var y = position.y;
        void DrawLine(string text, string text2 = "")
        {
            EditorGUI.LabelField(new Rect(position.x, y, position.width, 20), text, text2);
            y += LineHeight;
        }

        void DrawLineI(string text, int text2)
        {
            DrawLine(text, text2.ToString());
        }
        bool Button(string text)
        {
            var ret = GUI.Button(new Rect(position.x, y, position.width, 20), text);
            y += LineHeight;
            return ret;
        }

        EditorGUI.BeginProperty(position, label, property);
        if (net == null)
        {
            DrawLine("IsblNet is not active.");
            DrawLine("");
            DrawLine("IsblPersistentData");
            DrawLine(".DataDirectory", IsblPersistentDataSaver<IsblPersistentData>.DataDirectory);
            DrawLine(".LogLocalData", IsblPersistentData.Instance.LogLocalData.ToString());
            DrawLine(".GetLatestConnection()");
            var data = IsblPersistentData.Instance.GetLatestConnection();
            DrawLine("    PeerId", data.PeerId.ToString());
            DrawLine("    PeerIdToken", $"({data.PeerIdToken.Length}){new string('•', data.PeerIdToken.Length)}");
            DrawLine("    SocketUrl", data.SocketUrl);
        }
        else
        {
            DrawLine("SocketUrl", net.SocketUrl ?? "no socket");
            const int TextWidth = 75;
            _url = EditorGUI.TextField(new Rect(position.x, y, position.width - TextWidth, 20), "New URL", _url);
            if (GUI.Button(new Rect(position.width - TextWidth + 10, y, TextWidth, 20), "change"))
                net.SocketUrl = _url;
            y += LineHeight;

            DrawLine("Socket.State", net.UnityEditorOnlyDebug.State.ToString());
            DrawLine("SelfId", net.SelfId.ToString());
            DrawLine("Last Successful Message", net.UnityEditorOnlyDebug.LastSuccessfulMessage.ToLongTimeString());
            DrawLineI("Data Message Size", net.Stats.MessageSize);
            DrawLineI("    Gzip Size", net.Stats.MessageSizeGzip);
            DrawLineI("    Brotli Size", net.Stats.MessageSizeBrotli);
            DrawLineI("    Max Brotli", net.Stats.MessageSizeBrotliMax);
            foreach (var line in SerializeIsblNet(net)) DrawLine(line.Key, line.Value);
            DrawLine("Last Redraw", System.DateTime.Now.ToLongTimeString());
        }
        EditorGUI.EndProperty();
        if (net != null && Button("Simulate Disconnect"))
        {
            net?.UnityEditorOnlyDebug.SimulateDisconnect();
        }
    }

    List<KeyValuePair<string, string>> SerializeIsblNet(IsblNet net)
    {
        List<KeyValuePair<string, string>> val = new();
        if (net == null) return val;

        var jsonString = JsonSerializer.Serialize(net.ServerState, new JsonSerializerOptions { WriteIndented = true });

        var regex = new Regex(@"{\s*(""[xyz]"":\s*[0-9]+(\.[0-9]+)?),\s*(""[xyz]"":\s*[0-9]+(\.[0-9]+)?),\s*(""[xyz]"":\s*[0-9]+(\.[0-9]+)?)\s*}");
        jsonString = regex.Replace(jsonString, "{ $1, $3, $5 }");

        foreach (var line in jsonString.Split(System.Environment.NewLine))
        {
            var index = line.IndexOf(":");
            if (line.Trim() == "}," || line.Trim() == "}" || line == "{") continue;
            if (index < 0)
            {
                val.Add(new(line, ""));
            }
            else
            {
                var key = line[..index];
                var value = line[(index + 2)..];
                var trimmedKey = key.TrimStart();
                var indent = key.Length - trimmedKey.Length;
                if (trimmedKey.StartsWith("\"") && key.EndsWith("\""))
                {
                    key = key[(key.Length - trimmedKey.Length + 1)..(key.Length - 1)];
                }
                if (value.EndsWith(",")) value = value[0..(value.Length - 1)];
                if (value == "{") value = "";
                val.Add(new(string.Concat(Enumerable.Repeat(" |  ", indent / 2 - 1)) + key, value));
            }
        }
        return val;
    }
}
