using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;
using System.Linq;
using UnityEngine.XR.OpenXR;

public class IsblLocalXRDeviceManager : MonoBehaviour
{
    readonly List<IsblTrackedPoseDriver> _devices = new();

    void OnEnable()
    {
        InputDevices.deviceConnected += DeviceConnected;
        InputDevices.deviceDisconnected += DeviceDisconnected;

        var currentDevices = new List<InputDevice>();
        InputDevices.GetDevices(currentDevices);
        _devices.AddRange(currentDevices.Select(d => CreateDriver(d)).Where(d => d != null));

        List<SubsystemDescriptor> subsystemDescriptors = new();
        SubsystemManager.GetSubsystemDescriptors(subsystemDescriptors);

        static string SelectVersion(string ext)
        {
            var version = OpenXRRuntime.GetExtensionVersion(ext);
            return version == 1 ? ext : $"{ext} v{version}";
        }

        Debug.Log($@"OpenXR info
        OpenXRRuntime.name: {OpenXRRuntime.name}
        OpenXRRuntime.apiVersion: {OpenXRRuntime.apiVersion}
        OpenXRRuntime.pluginVersion: {OpenXRRuntime.pluginVersion}
        OpenXRRuntime.version: {OpenXRRuntime.version}
        Available extensions: {string.Join(", ", OpenXRRuntime.GetAvailableExtensions().Select(SelectVersion))}
        Enabled extensions: {string.Join(", ", OpenXRRuntime.GetEnabledExtensions().Select(SelectVersion))}
        Subsystems: {(subsystemDescriptors.Count < 1 ? "none" : string.Join(", ", subsystemDescriptors.ConvertAll(s => s.id)))}
        ");
    }

    IsblTrackedPoseDriver CreateDriver(InputDevice device)
    {
        IsblTrackedPoseDriver driver;
        if ((device.characteristics & InputDeviceCharacteristics.HeadMounted) != InputDeviceCharacteristics.None)
        {
            driver = Camera.main.GetComponent<IsblTrackedPoseDriver>();
        }
        else
        {
            var go = new GameObject($"{device.characteristics & (InputDeviceCharacteristics.Left | InputDeviceCharacteristics.Right)} {device.name}");
            driver = go.AddComponent<IsblTrackedPoseDriver>();
            driver.transform.parent = transform;
        }
        if (driver == null) return null;
        driver.LocalDevice = new IsblXRDevice(device);

        #region add to IsblNet
        var net = IsblNet.Instance;
        net?.LocalState.Devices.Add(driver.LocalDevice.LocallyUniqueId, driver.NetDevice);
        if (net == null) Debug.LogWarning("IsblNet is null");
        #endregion

        return driver;
    }

    void OnDisable()
    {
        InputDevices.deviceConnected -= DeviceConnected;
        InputDevices.deviceDisconnected -= DeviceDisconnected;
        _devices.Clear();
    }

    void DeviceDisconnected(InputDevice obj)
    {
        _devices.RemoveAll(d =>
        {
            if (d.LocalDevice.Device == obj)
            {
                IsblNet.Instance?.LocalState.Devices.Remove(d.LocalDevice.LocallyUniqueId);
                return true;
            }
            return false;
        });
    }

    void DeviceConnected(InputDevice obj)
    {
        List<InputFeatureUsage> usages = new();
        obj.TryGetFeatureUsages(usages);
        var text = string.Join("\n", from u in usages select $"{u.name}: {u.type}");
        Debug.Log($"Input device connected {obj.name}\n{obj.characteristics}\n{text}");

        var driver = CreateDriver(obj);
        if (driver != null) _devices.Add(driver);
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
