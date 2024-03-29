using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;
using System.Linq;
using UnityEngine.XR.OpenXR;
using System;

public class IsblLocalXRDeviceManager : MonoBehaviour
{
    public readonly List<IsblTrackedPoseDriver> Devices = new();
    public bool DeviceInfoChanged = false;

    void OnEnable()
    {


        InputDevices.deviceConnected += DeviceConnected;
        InputDevices.deviceDisconnected += DeviceDisconnected;
        InputDevices.deviceConfigChanged += DeviceConfigChanged;

        var currentDevices = new List<InputDevice>();
        InputDevices.GetDevices(currentDevices);
        foreach (var device in currentDevices) DeviceConnected(device);
        DeviceInfoChanged = true;

        List<SubsystemDescriptor> subsystemDescriptors = new();
        SubsystemManager.GetSubsystemDescriptors(subsystemDescriptors);

        static string SelectVersion(string ext)
        {
            var version = OpenXRRuntime.GetExtensionVersion(ext);
            return version == 1 ? ext : $"{ext} v{version}";
        }
        var features = OpenXRSettings.ActiveBuildTargetInstance.GetFeatures();
        Debug.Log($@"OpenXR info
        OpenXRRuntime.name: {OpenXRRuntime.name}
        OpenXRRuntime.apiVersion: {OpenXRRuntime.apiVersion}
        OpenXRRuntime.pluginVersion: {OpenXRRuntime.pluginVersion}
        OpenXRRuntime.version: {OpenXRRuntime.version}
        Available extensions: {string.Join(", ", OpenXRRuntime.GetAvailableExtensions().Select(SelectVersion))}
        Enabled extensions: {string.Join(", ", OpenXRRuntime.GetEnabledExtensions().Select(SelectVersion))}
        Subsystems: {(subsystemDescriptors.Count < 1 ? "none" : string.Join(", ", subsystemDescriptors.ConvertAll(s => s.id)))}
        Features: {string.Join(", ", features.Select(f => f.name + (f.enabled ? "" : " (disabled)")))}
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
        driver.NetDevice.DeviceInfoChanged += OnDeviceInfoChanged;

        return driver;
    }

    void OnDeviceInfoChanged()
    {
        DeviceInfoChanged = true;
    }

    void OnDisable()
    {
        InputDevices.deviceConnected -= DeviceConnected;
        InputDevices.deviceDisconnected -= DeviceDisconnected;
        InputDevices.deviceConfigChanged -= DeviceConfigChanged;

        while (Devices.Count > 0)
            DeviceDisconnected(Devices[0].LocalDevice.Device);

    }

    void DeviceDisconnected(InputDevice obj)
    {
        Utils.Log($"Input device disconnected {obj.name}\n{obj.characteristics}");
        var index = Devices.FindIndex(d => d.LocalDevice.Device == obj);
        if (index >= 0) Devices.RemoveAt(index);

        DeviceInfoChanged = true;
    }

    void DeviceConfigChanged(InputDevice device)
    {
        DeviceDisconnected(device);
        DeviceConnected(device);
    }

    static string TrackingOriginModeFlagsToString(TrackingOriginModeFlags flags)
    {
        var supportedModesString = "";
        void CheckMode(TrackingOriginModeFlags flag)
        {
            if ((flags & flag) != 0)
            {
                if (supportedModesString != "") supportedModesString += ", ";
                supportedModesString += flag;
            }
        }
        CheckMode(TrackingOriginModeFlags.Device);
        CheckMode(TrackingOriginModeFlags.Floor);
        CheckMode(TrackingOriginModeFlags.TrackingReference);
        CheckMode(TrackingOriginModeFlags.Unbounded);
        if (flags == TrackingOriginModeFlags.Unknown) supportedModesString = "Unknown";
        return supportedModesString;
    }

    void DeviceConnected(InputDevice obj)
    {
        List<InputFeatureUsage> usages = new();
        obj.TryGetFeatureUsages(usages);
        var text = string.Join("\n", from u in usages select $"    {u.name}: {u.type}");
        var supportedModesString = TrackingOriginModeFlagsToString(obj.subsystem.GetSupportedTrackingOriginModes());

        Utils.Log($"Input device connected {obj.name}\n    {obj.characteristics}\n{text}\n    TrackingOriginMode: {obj.subsystem.GetTrackingOriginMode()}\n    SupportedModes: {TrackingOriginModeFlagsToString(obj.subsystem.GetSupportedTrackingOriginModes())}");

        var driver = CreateDriver(obj);
        if (driver != null) Devices.Add(driver);

        DeviceInfoChanged = true;
    }


}
