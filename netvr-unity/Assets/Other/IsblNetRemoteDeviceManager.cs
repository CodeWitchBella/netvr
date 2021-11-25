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
            for (int i = 0; i < peer.Value.Devices.Count; ++i)
                SyncDevice(peer.Value.Id * 1000 + i, peer.Value.Devices[i]);
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

    void SyncDevice(int id, IsblStaticXRDevice deviceData)
    {
        var device = _remoteDevices.GetValueOrDefault(id, null);
        if (device == null)
        {
            var go = new GameObject($"Synced {id / 3}:{id % 3}");
            go.transform.parent = transform;
            device = go.AddComponent<IsblNetRemoteDevice>();
            device.Id = id;
            go.AddComponent<IsblTrackedPoseDriver>();
            _remoteDevices.Add(id, device);
        }
        device.Visited = true;
        var driver = device.GetComponent<IsblTrackedPoseDriver>();
        if (deviceData != null)
            driver.NetDevice = deviceData;
    }
}
