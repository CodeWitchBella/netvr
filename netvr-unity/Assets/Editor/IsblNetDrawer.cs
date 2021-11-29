using UnityEngine;
using UnityEditor;

[CustomPropertyDrawer(typeof(IsblNetComponent.SelfPropertyAttribute))]
public class IsblNetDrawer : PropertyDrawer
{
    const float LineHeight = 20;
    string _url;

    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        return LineHeight * 24;
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
            DrawLine(".DataPath", IsblPersistentData.DataPath);
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
            DrawLine("PeerId", net.LocalState.Id.ToString());
            DrawLine("Initialized", net.LocalState.Initialized.ToString());
            DrawLine("Last Successful Message", net.UnityEditorOnlyDebug.LastSuccessfulMessage.ToLongTimeString());
            DrawLineI("Data Message Size", net.Stats.MessageSize);
            DrawLineI("    Gzip Size", net.Stats.MessageSizeGzip);
            DrawLineI("    Brotli Size", net.Stats.MessageSizeBrotli);
            DrawLineI("    Max Brotli", net.Stats.MessageSizeBrotliMax);
            DrawLine("Peer count", net.OtherStates.Count.ToString());
        }
        EditorGUI.EndProperty();
        if (net != null && Button("Simulate Disconnect"))
        {
            net?.UnityEditorOnlyDebug.SimulateDisconnect();
        }
    }
}
