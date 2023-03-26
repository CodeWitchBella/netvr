using System.Collections.Generic;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.XR.OpenXR;


namespace Isbl.NetVR
{

public class IsblRemoteDeviceManager : MonoBehaviour
{
    readonly Dictionary<UInt32, Transform> _devices = new();

    void Update()
    {
        var feature = IsblXRFeature.Instance;
        var instance = feature.XrInstance;
        if (feature == null || instance == 0) {
            foreach(var entry in _devices)
                Destroy(entry.Value);
            _devices.Clear();
            return;
        }
        var remoteDevices = feature.RustLib.ReadRemoteDevices(instance);
        Debug.Log($"Devices: {remoteDevices.Length}");

        var toRemove = _devices.Where(device => {
            if (Array.FindIndex(remoteDevices, f => f.id == device.Key) < 0) {
                Destroy(device.Value);
                return true;
            }
            return false;
        }).Select(device => device.Key).ToArray();
        foreach (var key in toRemove) _devices.Remove(key);

        foreach (var remoteDevice in remoteDevices) {
            Transform device;
            if (!_devices.TryGetValue(remoteDevice.id, out device)) {
                var obj = new GameObject($"Device {remoteDevice.id}");
                device = obj.transform;
                device.parent = transform;
                _devices.Add(remoteDevice.id, device);
            }
            
            // TODO: sync position
        }
        //foreach()
    }
}

}
