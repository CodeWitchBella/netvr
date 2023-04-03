using System.Collections.Generic;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.XR.OpenXR;


namespace Isbl.NetVR
{

    public class IsblRemoteDeviceManager : MonoBehaviour
    {
        readonly Dictionary<UInt32, IsblRemoteDevice> _devices = new();

        void Update()
        {
            var feature = IsblXRFeature.Instance;
            var instance = feature.XrInstance;
            if (feature == null || instance == 0)
            {
                foreach (var entry in _devices)
                    Destroy(entry.Value.gameObject);
                _devices.Clear();
                return;
            }
            var remoteDevices = feature.RustLib.ReadRemoteDevices(instance);
            Debug.Log($"Devices: {remoteDevices.Length}");

            var toRemove = _devices.Where(device =>
            {
                if (Array.FindIndex(remoteDevices, f => f.id == device.Key) < 0)
                {
                    Destroy(device.Value.gameObject);
                    return true;
                }
                return false;
            }).Select(device => device.Key).ToArray();
            foreach (var key in toRemove) _devices.Remove(key);

            foreach (var remoteDevice in remoteDevices)
            {
                IsblRemoteDevice device;
                if (!_devices.TryGetValue(remoteDevice.id, out device))
                {
                    var obj = new GameObject($"Device {remoteDevice.id}");
                    device = obj.AddComponent<IsblRemoteDevice>();
                    device.transform.parent = transform;
                    device.Id = remoteDevice.id;
                    _devices.Add(remoteDevice.id, device);
                }
                device.transform.position = remoteDevice.pos;
                device.transform.rotation = remoteDevice.quat;
            }
            //foreach()
        }
    }

}
