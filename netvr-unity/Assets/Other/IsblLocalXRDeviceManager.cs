using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;
using System.Linq;

public class IsblLocalXRDeviceManager : MonoBehaviour
{
    readonly List<IsblXRDevice> _devices = new();

    void OnEnable()
    {
        InputDevices.deviceConnected += DeviceConnected;
        InputDevices.deviceDisconnected += DeviceDisconnected;

        var currentDevices = new List<InputDevice>();
        InputDevices.GetDevices(currentDevices);
        _devices.AddRange(currentDevices.Select(d => new IsblXRDevice(d)));
    }

    void OnDisable()
    {
        InputDevices.deviceConnected -= DeviceConnected;
        InputDevices.deviceDisconnected -= DeviceDisconnected;
        _devices.Clear();
    }

    void DeviceDisconnected(InputDevice obj)
    {
        _devices.RemoveAll(d => d.Device == obj);
    }

    void DeviceConnected(InputDevice obj)
    {
        List<InputFeatureUsage> usages = new();
        obj.TryGetFeatureUsages(usages);
        var text = string.Join("\n", from u in usages select $"{u.name}: {u.type}");
        Debug.Log($"Input device connected {obj.name}\n{obj.characteristics}\n{text}");

        _devices.Add(new(obj));
    }

    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;
        transform.localPosition = net.LocalState.CalibrationPosition;
        transform.localRotation = net.LocalState.CalibrationRotation;
        transform.localScale = net.LocalState.CalibrationScale;
    }
}
