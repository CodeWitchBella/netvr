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
        IsblNet.Instance.DeviceManager = this;

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

        var net = IsblNet.Instance;
        while (Devices.Count > 0)
            DeviceDisconnected(Devices[0].LocalDevice.Device);

        if (IsblNet.Instance.DeviceManager == this) IsblNet.Instance.DeviceManager = null;
    }

    void DeviceDisconnected(InputDevice obj)
    {
        Debug.Log($"Input device disconnected {obj.name}\n{obj.characteristics}");
        var index = Devices.FindIndex(d => d.LocalDevice.Device == obj);
        if (index >= 0) Devices.RemoveAt(index);

        DeviceInfoChanged = true;
    }

    void DeviceConfigChanged(InputDevice device)
    {
        DeviceDisconnected(device);
        DeviceConnected(device);
    }

    void DeviceConnected(InputDevice obj)
    {
        List<InputFeatureUsage> usages = new();
        obj.TryGetFeatureUsages(usages);
        var text = string.Join("\n", from u in usages select $"{u.name}: {u.type}");
        var supportedModesString = "";
        var supportedModes = obj.subsystem.GetSupportedTrackingOriginModes();
        void CheckMode(TrackingOriginModeFlags flag)
        {
            if ((supportedModes & flag) != 0)
            {
                if (supportedModesString != "") supportedModesString += ", ";
                supportedModesString += flag;
            }
        }
        CheckMode(TrackingOriginModeFlags.Device);
        CheckMode(TrackingOriginModeFlags.Floor);
        CheckMode(TrackingOriginModeFlags.TrackingReference);
        CheckMode(TrackingOriginModeFlags.Unbounded);
        if (supportedModes == TrackingOriginModeFlags.Unknown) supportedModesString = "Unknown";

        Debug.Log($"Input device connected {obj.name}\n{obj.characteristics}\n{text}\nTrackingOriginMode: {obj.subsystem.GetTrackingOriginMode()}\n  SupportedModes: {obj.subsystem.GetSupportedTrackingOriginModes()}");

        var driver = CreateDriver(obj);
        if (driver != null) Devices.Add(driver);

        DeviceInfoChanged = true;
    }

    void Update()
    {
        var net = IsblNet.Instance;
        if (net == null) return;

        // TODO: only do this on calibration config change
        if (net.ServerState.Clients.TryGetValue(net.SelfId, out var self))
        {
            transform.localPosition = self.Calibration.Translate;
            transform.localRotation = self.Calibration.Rotate;
            transform.localScale = self.Calibration.Scale;
        }
    }

    public bool TryFindDevice(UInt16 id, out IsblTrackedPoseDriver outDevice)
    {
        var deviceIdx = Devices.FindIndex(d => d.NetDevice.LocallyUniqueId == id);
        if (deviceIdx >= 0)
        {
            outDevice = Devices[deviceIdx];
            return true;
        }
        outDevice = new();
        return false;
    }

    public int CalculateSerializationSize()
    {
        return Isbl.NetUtils.Count7BitEncodedIntBytes(Devices.Count(d => d.NetDevice.HasData)) /* Device count */
            + (from d in Devices where d.NetDevice.HasData select d.NetDevice.CalculateSerializationSize()).Sum() /* devices array */;
    }
}
