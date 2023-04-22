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
        public GameObject DevicePrefab;

        void Update()
        {
            var feature = IsblXRFeature.Instance;
            var instance = feature.XrInstance;
            var session = feature.XrSession;
            if (feature == null || instance == 0 || session == 0)
            {
                foreach (var entry in _devices)
                    Destroy(entry.Value.gameObject);
                _devices.Clear();
                return;
            }
            var remoteDevices = feature.RPC.ReadRemoteDevices(new(instance, session)).devices.ToArray();
            //Debug.Log($"Devices: {remoteDevices.Length}");

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
                    device = SpawnDevice(remoteDevice.id);
                    _devices.Add(remoteDevice.id, device);
                }
                device.transform.position = Convertor.Vector3(remoteDevice.pos);
                device.transform.rotation = Convertor.Quaternion(remoteDevice.rot);
            }
            //foreach()
        }

        IsblRemoteDevice SpawnDevice(UInt32 id)
        {
            var obj = DevicePrefab == null ? new GameObject() : Instantiate(DevicePrefab);
            obj.name = $"Device {id}";
            var device = obj.GetComponent<IsblRemoteDevice>();
            if (device == null) device = obj.AddComponent<IsblRemoteDevice>();
            obj.transform.parent = transform;
            device.Id = id;
            return device;
        }
    }
}
