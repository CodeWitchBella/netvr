using UnityEngine;
using UnityEditor;
using System.Collections.Generic;

[CustomPropertyDrawer(typeof(IsblNetComponent.SelfPropertyAttribute))]
public class IsblNetDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    string _url;

    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        var net = IsblNet.Instance;
        return LineHeight * (24 + SerializeIsblNet(net).Count);
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        var net = IsblNet.Instance;

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
            var lastIndent = 0;
            foreach (var line in SerializeIsblNet(net))
            {
                var key = line.Key.TrimStart();
                var indent = line.Key.Length - key.Length;
                for (; lastIndent < indent; lastIndent++) EditorGUI.indentLevel++;
                for (; lastIndent > indent; lastIndent--) EditorGUI.indentLevel--;
                DrawLine(line.Key, line.Value);
            }
            for (; lastIndent > 0; lastIndent--) EditorGUI.indentLevel--;

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

        static string Vec3ToString(Vector3 v) { return $"{{ x: {v.x}, y: {v.y}, z: {v.z} }}"; }

        val.Add(new("ServerState.Clients", ""));
        int disconnectedCount = 0;
        foreach (var client in net.ServerState.Clients)
        {
            if (!client.Value.Connected)
            {
                disconnectedCount++;
                continue;
            }
            val.Add(new(" " + client.Key, ""));
            val.Add(new("  Translate", Vec3ToString(client.Value.Calibration.Translate)));
            val.Add(new("  Rotate", Vec3ToString(client.Value.Calibration.Rotate.eulerAngles)));
            val.Add(new("  Scale", Vec3ToString(client.Value.Calibration.Scale)));
            if (client.Value.Devices != null)
                val.Add(new("  Devices", $"array of length {client.Value.Devices.Count}"));
        }
        val.Add(new($" + {disconnectedCount} disconnected clients", ""));
        return val;
    }

}
