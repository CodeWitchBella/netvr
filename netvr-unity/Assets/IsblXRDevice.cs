using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;

/*
Usages available:
                               Vive | Quest2
DeviceAngularVelocity(Vector3)  ✅   ✅
DevicePosition(Vector3)         ✅   ✅
DeviceRotation(Quaternion)      ✅   ✅
DeviceVelocity(Vector3)         ✅   ✅
Grip(float)                     ✅   ✅
GripButton(bool)                ✅   ✅
IsTracked(bool)                 ✅   ✅
Menu(bool)                      ❌   ✅
MenuButton(bool)                ✅   ❌
PointerAngularVelocity(Vector3) ✅   ✅
PointerPosition(Vector3)        ✅   ✅
PointerRotation(Quaternion)     ✅   ✅
PointerVelocity(Vector3)        ✅   ✅
Primary2DAxis(Vector2)          ✅   ✅
Primary2DAxisClick(bool)        ✅   ✅
Primary2DAxisTouch(bool)        ✅   ✅
PrimaryButton(bool)             ❌   ✅
PrimaryTouch(bool)              ❌   ✅
SecondaryButton(bool)           ❌   ✅
SecondaryTouch(bool)            ❌   ✅
SystemButton(bool)              ✅   ❌
TrackingState(uint)             ✅   ✅
Trigger(float)                  ✅   ✅
TriggerButton(bool)             ✅   ✅
TriggerTouch(bool)              ❌   ✅
*/

public class IsblXRDevice
{
    public InputDevice Device { get; }

    InputFeatureUsage<Vector3> _deviceAngularVelocity;
    public Vector3 DeviceAngularVelocity { get { return Read(_deviceAngularVelocity); } }
    InputFeatureUsage<Vector3> _devicePosition;
    public Vector3 DevicePosition { get { return Read(_devicePosition); } }
    InputFeatureUsage<Quaternion> _deviceRotation;
    public Quaternion DeviceRotation { get { return Read(_deviceRotation); } }
    InputFeatureUsage<Vector3> _deviceVelocity;
    public Vector3 DeviceVelocity { get { return Read(_deviceVelocity); } }
    InputFeatureUsage<float> _grip;
    public float Grip { get { return Read(_grip); } }
    InputFeatureUsage<bool> _gripButton;
    public bool GripButton { get { return Read(_gripButton); } }
    InputFeatureUsage<bool> _isTracked;
    public bool IsTracked { get { return Read(_isTracked); } }
    // Note: I merged Oculus Touch's MenuButton and Vive's Menu into MenuButton only
    InputFeatureUsage<bool> _menuButton;
    public bool MenuButton { get { return Read(_menuButton); } }
    InputFeatureUsage<Vector3> _pointerAngularVelocity;
    public Vector3 PointerAngularVelocity { get { return Read(_pointerAngularVelocity); } }
    InputFeatureUsage<Vector3> _pointerPosition;
    public Vector3 PointerPosition { get { return Read(_pointerPosition); } }
    InputFeatureUsage<Quaternion> _pointerRotation;
    public Quaternion PointerRotation { get { return Read(_pointerRotation); } }
    InputFeatureUsage<Vector3> _pointerVelocity;
    public Vector3 PointerVelocity { get { return Read(_pointerVelocity); } }
    InputFeatureUsage<Vector2> _primary2DAxis;
    public Vector2 Primary2DAxis { get { return Read(_primary2DAxis); } }
    InputFeatureUsage<bool> _primary2DAxisClick;
    public bool Primary2DAxisClick { get { return Read(_primary2DAxisClick); } }
    InputFeatureUsage<bool> _primary2DAxisTouch;
    public bool Primary2DAxisTouch { get { return Read(_primary2DAxisTouch); } }
    InputFeatureUsage<bool> _primaryButton;
    public bool PrimaryButton { get { return Read(_primaryButton); } }
    InputFeatureUsage<bool> _primaryTouch;
    public bool PrimaryTouch { get { return Read(_primaryTouch); } }
    InputFeatureUsage<bool> _secondaryButton;
    public bool SecondaryButton { get { return Read(_secondaryButton); } }
    InputFeatureUsage<bool> _secondaryTouch;
    public bool SecondaryTouch { get { return Read(_secondaryTouch); } }
    InputFeatureUsage<bool> _systemButton;
    public bool SystemButton { get { return Read(_systemButton); } }
    InputFeatureUsage<uint> _trackingState;
    public uint TrackingState { get { return Read(_trackingState); } }
    InputFeatureUsage<float> _trigger;
    public float Trigger { get { return Read(_trigger); } }
    InputFeatureUsage<bool> _triggerButton;
    public bool TriggerButton { get { return Read(_triggerButton); } }
    InputFeatureUsage<bool> _triggerTouch;
    public bool TriggerTouch { get { return Read(_triggerTouch); } }
    // ADD_FEATURE step 1: two lines above this line
    public IsblXRDevice(InputDevice device)
    {
        Device = device;
        var featureUsages = new List<InputFeatureUsage>();
        if (Device.TryGetFeatureUsages(featureUsages))
        {
            Debug.Log($"{device.name}: {string.Join(" ", featureUsages.ConvertAll(usage => $"{usage.name}({usage.type})"))}");
            foreach (var usage in featureUsages)
            {
                if (!InitUsage(usage))
                {
                    Debug.Log($"Unknown usage {usage.name} of type {usage.type} on {device.name}");
                }
            }
        }
    }

    bool InitUsage(InputFeatureUsage usage)
    {
        // SteamVR with Quest 2 Link: Primary2DAxis Grip GripButton Menu PrimaryButton PrimaryTouch SecondaryButton SecondaryTouch Trigger TriggerButton TriggerTouch Primary2DAxisClick Primary2DAxisTouch IsTracked TrackingState DevicePosition DeviceRotation DeviceVelocity DeviceAngularVelocity IsTracked TrackingState PointerPosition PointerRotation PointerVelocity PointerAngularVelocity

        if (usage.name == "DevicePosition") _devicePosition = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "DeviceRotation") _deviceRotation = new InputFeatureUsage<Quaternion>(usage.name);
        else if (usage.name == "DeviceAngularVelocity") _deviceAngularVelocity = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "DevicePosition") _devicePosition = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "DeviceRotation") _deviceRotation = new InputFeatureUsage<Quaternion>(usage.name);
        else if (usage.name == "DeviceVelocity") _deviceVelocity = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "Grip") _grip = new InputFeatureUsage<float>(usage.name);
        else if (usage.name == "GripButton") _gripButton = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "IsTracked") _isTracked = new InputFeatureUsage<bool>(usage.name);
        // Note: I merged Oculus Touch's MenuButton and Vive's Menu into MenuButton only
        else if (usage.name == "Menu" || usage.name == "MenuButton") _menuButton = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "PointerAngularVelocity") _pointerAngularVelocity = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "PointerPosition") _pointerPosition = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "PointerRotation") _pointerRotation = new InputFeatureUsage<Quaternion>(usage.name);
        else if (usage.name == "PointerVelocity") _pointerVelocity = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "Primary2DAxis") _primary2DAxis = new InputFeatureUsage<Vector2>(usage.name);
        else if (usage.name == "Primary2DAxisClick") _primary2DAxisClick = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "Primary2DAxisTouch") _primary2DAxisTouch = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "PrimaryButton") _primaryButton = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "PrimaryTouch") _primaryTouch = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "SecondaryButton") _secondaryButton = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "SecondaryTouch") _secondaryTouch = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "SystemButton") _systemButton = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "TrackingState") _trackingState = new InputFeatureUsage<uint>(usage.name);
        else if (usage.name == "Trigger") _trigger = new InputFeatureUsage<float>(usage.name);
        else if (usage.name == "TriggerButton") _triggerButton = new InputFeatureUsage<bool>(usage.name);
        else if (usage.name == "TriggerTouch") _triggerTouch = new InputFeatureUsage<bool>(usage.name);
        // ADD_FEATURE step 2: one line above this line
        else return false;
        return true;
    }

    Vector2 Read(InputFeatureUsage<Vector2>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        Vector2 value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }
    Vector3 Read(InputFeatureUsage<Vector3>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        Vector3 value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }
    Quaternion Read(InputFeatureUsage<Quaternion>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        Quaternion value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }
    bool Read(InputFeatureUsage<bool>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        bool value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }
    float Read(InputFeatureUsage<float>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        float value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }
    uint Read(InputFeatureUsage<uint>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        uint value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }
    // ADD_FEATURE step 3: if new type is needed copy add new variant of Read method
}
