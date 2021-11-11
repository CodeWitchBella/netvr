using UnityEngine;
using UnityEditor;

[CustomPropertyDrawer(typeof(IsblNetComponent.SelfPropertyAttribute))]
public class IsblNetDrawer : PropertyDrawer
{
    const float LineHeight = 20;
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

        EditorGUI.BeginProperty(position, label, property);
        if (net == null)
        {
            DrawLine("IsblNet is not active.");
            DrawLine("ServerPort", IsblNet.ServerPort.ToString());
        }
        else
        {
            DrawLine("ServerPort", IsblNet.ServerPort.ToString());
            DrawLine("TcpConnected", net.TcpConnected ? "Yes" : "No");
        }
        EditorGUI.EndProperty();
    }
}