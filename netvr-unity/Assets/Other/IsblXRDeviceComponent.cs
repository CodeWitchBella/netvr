using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;

/// <summary>
/// Keeps track of devices at specified XRNode and provides first connected,
/// tracking device to IsblTrackedPoseDriver for updates.
/// </summary>
///
/// It always returns first connected controller, but also makes sure that if
/// a controller disconnects and reconnects that it does not kick out previously
/// used controller.
///
/// Example:
/// A connects, returns A
/// B connects, returns A
/// A disconnects, returns B
/// A connects, still returns B
[RequireComponent(typeof(IsblTrackedPoseDriver))]
public class IsblXRDeviceComponent : MonoBehaviour
{
    public XRNode Node = XRNode.LeftHand;

    readonly List<IsblXRDevice> _devices = new();
    public IsblXRDevice Device
    {
        get
        {
            int index = _devices.FindIndex(d => d.TrackingState != 0);
            if (index < 0) return null;
            if (index != 0)
            {
                // move the object to the start so that even if other conroller
                // reconnects we still get the same one
                var dev = _devices[index];
                _devices.RemoveAt(index);
                _devices.Insert(0, dev);
            }
            return _devices[0];
        }
    }

    void OnEnable()
    {
        InputDevices.deviceConnected += DeviceConnected;
        InputDevices.deviceDisconnected += DeviceDisconnected;
        UpdateDeviceList();
    }

    void OnDisable()
    {
        InputDevices.deviceConnected -= DeviceConnected;
        InputDevices.deviceDisconnected -= DeviceDisconnected;
    }

    void DeviceDisconnected(InputDevice obj)
    {
        UpdateDeviceList();
    }

    void DeviceConnected(InputDevice obj)
    {
        UpdateDeviceList();
    }

    void UpdateDeviceList()
    {
        var currentDevices = new List<InputDevice>();
        InputDevices.GetDevicesAtXRNode(Node, currentDevices);

        // remove disconnected
        _devices.RemoveAll(d => !currentDevices.Contains(d.Device));

        // add new device
        foreach (var current in currentDevices)
        {
            if (_devices.Exists(d => d.Device == current)) continue;
            _devices.Add(new IsblXRDevice(current));
        }
    }
}
