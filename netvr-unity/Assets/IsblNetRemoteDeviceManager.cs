using System.Collections.Generic;
using UnityEngine;

public class IsblNetRemoteDeviceManager : MonoBehaviour
{
    readonly Dictionary<int, IsblNetRemoteDevice> _remoteDevices = new();

    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;
        foreach (var peer in _remoteDevices)
            peer.Value.Visited = false;

        foreach (var peer in net.OtherStates)
        {
            SyncDevice(peer.Id * 3, peer.Head);
            SyncDevice(peer.Id * 3 + 1, peer.Left);
            SyncDevice(peer.Id * 3 + 2, peer.Right);
        }

        // remove untracked
        List<int> toBeRemoved = new();
        foreach (var peer in _remoteDevices)
        {
            if (peer.Value.Visited) continue;
            Destroy(peer.Value.gameObject);
            toBeRemoved.Add(peer.Key);
        }
        foreach (var id in toBeRemoved) _remoteDevices.Remove(id);
    }

    void SyncDevice(int id, Isbl.NetDeviceData deviceData)
    {
        var device = _remoteDevices.GetValueOrDefault(id, null);
        if (device == null)
        {
            var go = new GameObject($"Synced {id / 3}:{id % 3}");
            go.transform.parent = transform;
            device = go.AddComponent<IsblNetRemoteDevice>();
            device.Id = id;
            device.Type = deviceData.Type;
            _remoteDevices.Add(id, device);
        }
        device.Visited = true;

        device.transform.localPosition = deviceData.Position;
        device.transform.localRotation = Quaternion.Euler(deviceData.Rotation);
    }
}