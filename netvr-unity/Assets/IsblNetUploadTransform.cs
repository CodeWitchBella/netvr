using UnityEngine;

public class IsblNetUploadTransform : MonoBehaviour
{
    public enum PartEnum
    {
        Left, Right, Head
    };

    public PartEnum Part;

    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;
        var data = new Isbl.NetDeviceData
        {
            Position = transform.localPosition,
            Rotation = transform.localEulerAngles
        };
        if (Part == PartEnum.Left) { net.NetState.Left = data; }
        else if (Part == PartEnum.Right) { net.NetState.Right = data; }
        else { net.NetState.Head = data; }
    }
}
