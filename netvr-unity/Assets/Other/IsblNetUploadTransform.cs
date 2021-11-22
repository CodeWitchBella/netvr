using UnityEngine;

public class IsblNetUploadTransform : MonoBehaviour
{
    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;
        net.NetState.Head = new Isbl.NetDeviceData
        {
            Position = transform.localPosition,
            Rotation = transform.localEulerAngles
        };
    }
}
