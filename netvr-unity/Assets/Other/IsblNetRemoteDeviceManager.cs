using System.Collections.Generic;
using UnityEngine;
using System.Linq;

public class IsblNetRemoteDeviceManager : MonoBehaviour
{
    readonly Dictionary<int, IsblNetRemoteClient> _remoteClients = new();

    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;

        var toRemove = _remoteClients.Where(client =>
        {
            if (!net.OtherStates.ContainsKey(client.Key))
            {
                Destroy(client.Value.gameObject);
                return true;
            }
            return false;
        }).Select(c => c.Key).ToArray();

        foreach (var key in toRemove) _remoteClients.Remove(key);

        foreach (var iter in net.OtherStates)
        {
            IsblNetRemoteClient client;
            if (_remoteClients.ContainsKey(iter.Key))
            {
                client = _remoteClients[iter.Key];
            }
            else
            {
                var go = new GameObject($"Client {iter.Key}");
                go.transform.parent = transform;
                client = go.AddComponent<IsblNetRemoteClient>();
                _remoteClients.Add(iter.Key, client);
            }

            SyncClient(iter.Value, client);
        }
    }

    static void SyncClient(Isbl.NetStateData netState, IsblNetRemoteClient remoteClient)
    {
        var toRemove = remoteClient.Devices.Where(device =>
        {
            if (!netState.Devices.Any(d => d.LocallyUniqueId == device.Key))
            {
                Destroy(device.Value.gameObject);
                return true;
            }
            return false;
        }).Select(c => c.Key).ToArray();
        foreach (var key in toRemove) remoteClient.Devices.Remove(key);

        foreach (var iter in netState.Devices)
        {
            IsblNetRemoteDevice device;
            if (remoteClient.Devices.ContainsKey(iter.LocallyUniqueId))
                device = remoteClient.Devices[iter.LocallyUniqueId];
            else
                device = CreateDevice(remoteClient, iter);
        }

        remoteClient.transform.localPosition = netState.CalibrationPosition;
        remoteClient.transform.localRotation = netState.CalibrationRotation;
        remoteClient.transform.localScale = netState.CalibrationScale;
    }

    static IsblNetRemoteDevice CreateDevice(IsblNetRemoteClient remoteClient, IsblStaticXRDevice iter)
    {
        var go = new GameObject($"Device {iter.LocallyUniqueId}");
        go.transform.parent = remoteClient.transform;
        var device = go.AddComponent<IsblNetRemoteDevice>();
        remoteClient.Devices.Add(iter.LocallyUniqueId, device);

        var driver = go.AddComponent<IsblTrackedPoseDriver>();
        driver.NetDevice = iter;
        return device;
    }
}
