using System.Collections.Generic;
using UnityEngine;
using System.Linq;
using System;

public class IsblNetRemoteDeviceManager : MonoBehaviour
{
    readonly Dictionary<UInt16, IsblNetRemoteClient> _remoteClients = new();

    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;

        var toRemove = _remoteClients.Where(client =>
        {
            if (!net.ServerState.Clients.TryGetValue(client.Key, out var serverClient) || !serverClient.Connected)
            {
                Destroy(client.Value.gameObject);
                return true;
            }
            return false;
        }).Select(c => c.Key).ToArray();

        foreach (var key in toRemove) _remoteClients.Remove(key);

        foreach (var iter in net.ServerState.Clients)
        {
            if (!iter.Value.Connected) continue;
            if (iter.Key == net.SelfId) continue;

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
#if UNITY_EDITOR
                client.Id = iter.Key;
#endif
                _remoteClients.Add(iter.Key, client);
            }

            SyncClient(net, iter.Key, iter.Value, client);
        }
    }

    static void SyncClient(IsblNet net, UInt16 id, Isbl.NetServerState.Client netState, IsblNetRemoteClient remoteClient)
    {
        if (!net.FastState.Clients.TryGetValue(id, out var fastStateDevices))
        {
            fastStateDevices = new();
            net.FastState.Clients.Add(id, fastStateDevices);
        }

        var toRemove = remoteClient.Devices.Where(device =>
        {
            if (!fastStateDevices.ContainsKey(device.Key))
            {
                Destroy(device.Value.gameObject);
                return true;
            }
            return false;
        }).Select(c => c.Key).ToArray();
        foreach (var key in toRemove) remoteClient.Devices.Remove(key);

        foreach (var iter in fastStateDevices)
        {
            IsblNetRemoteDevice device;
            if (remoteClient.Devices.ContainsKey(iter.Key))
                device = remoteClient.Devices[iter.Key];
            else
                device = CreateDevice(remoteClient, iter.Value.DeviceData);
        }

        remoteClient.transform.localPosition = netState.Calibration.Translate;
        remoteClient.transform.localRotation = netState.Calibration.Rotate;
        remoteClient.transform.localScale = netState.Calibration.Scale;
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
