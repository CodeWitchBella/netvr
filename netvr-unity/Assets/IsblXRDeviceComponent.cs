using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;

[RequireComponent(typeof(IsblTrackedPoseDriver))]
public class IsblXRDeviceComponent : MonoBehaviour
{
    public XRNode Node = XRNode.LeftHand;

    public IsblXRDevice Device { get; private set; }

    void OnEnable()
    {
        InputDevices.deviceConnected += DeviceConnected;
        InputDevices.deviceDisconnected += DeviceDisconnected;
        Device = null;
        InitializeIfNeeded();
    }

    void OnDisable()
    {
        InputDevices.deviceConnected -= DeviceConnected;
        InputDevices.deviceDisconnected -= DeviceDisconnected;
    }

    void DeviceDisconnected(InputDevice obj)
    {
        if (Device?.Device == obj) Device = null;
        InitializeIfNeeded();
    }

    void DeviceConnected(InputDevice obj)
    {
        InitializeIfNeeded();
    }

    void InitializeIfNeeded()
    {
        if (Device != null) return; // not needed

        // get device
        var devices = new List<InputDevice>();
        InputDevices.GetDevicesAtXRNode(Node, devices);
        if (devices.Count < 1) return;
        Device = new IsblXRDevice(devices[0]);
    }
}
