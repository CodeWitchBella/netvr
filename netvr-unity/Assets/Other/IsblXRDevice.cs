using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;

/*
Usages available:

*/

/// <summary>
/// Wrapper around XR.InputDevice which allows you to more conveniently read values
/// of various buttons/triggers etc.
/// </summary>
///
/// Only tested with Vive Wands and Oculus Quest 2 Touch controllers but there is
/// nothing that would prevent it working under other headsets provided that they
/// have the required buttons.
///
/// When button is missing from either Quest 2 or Vive Wands controller I provided
/// reasonable default (buttons always return false, because you can't press)
/// nonexistent button... Otherwise it throws an exception upon reading the value.
/// | Usage                           | Vive|Quest2|
/// |---------------------------------|-----|------|
/// | DeviceAngularVelocity(Vector3)  | ✅ |  ✅  |
/// | DevicePosition(Vector3)         | ✅ |  ✅  |
/// | DeviceRotation(Quaternion)      | ✅ |  ✅  |
/// | DeviceVelocity(Vector3)         | ✅ |  ✅  |
/// | Grip(float)                     | ✅ |  ✅  |
/// | GripButton(bool)                | ✅ |  ✅  |
/// | IsTracked(bool)                 | ✅ |  ✅  |
/// | Menu(bool)                      | ❌ |  ✅  |
/// | MenuButton(bool)                | ✅ |  ❌  |
/// | PointerAngularVelocity(Vector3) | ✅ |  ✅  |
/// | PointerPosition(Vector3)        | ✅ |  ✅  |
/// | PointerRotation(Quaternion)     | ✅ |  ✅  |
/// | PointerVelocity(Vector3)        | ✅ |  ✅  |
/// | Primary2DAxis(Vector2)          | ✅ |  ✅  |
/// | Primary2DAxisClick(bool)        | ✅ |  ✅  |
/// | Primary2DAxisTouch(bool)        | ✅ |  ✅  |
/// | PrimaryButton(bool)             | ❌ |  ✅  |
/// | PrimaryTouch(bool)              | ❌ |  ✅  |
/// | SecondaryButton(bool)           | ❌ |  ✅  |
/// | SecondaryTouch(bool)            | ❌ |  ✅  |
/// | SystemButton(bool)              | ✅ |  ❌  |
/// | TrackingState(uint)             | ✅ |  ✅  |
/// | Trigger(float)                  | ✅ |  ✅  |
/// | TriggerButton(bool)             | ✅ |  ✅  |
/// | TriggerTouch(bool)              | ❌ |  ✅  |
public class IsblXRDevice
{
    public InputDevice Device { get; }

    InputFeatureUsage<Vector3>? _deviceAngularVelocity;
    /// <summary>Rate of change in DeviceRotation</summary>
    public Vector3 DeviceAngularVelocity { get { return Read(_deviceAngularVelocity); } }
    InputFeatureUsage<Vector3>? _devicePosition;
    /// <summary>Position of this XR Device in tracking space</summary>
    public Vector3 DevicePosition { get { return Read(_devicePosition); } }
    InputFeatureUsage<Quaternion>? _deviceRotation;
    /// <summary>Rotation of this XR Device relative to tracking space</summary>
    public Quaternion DeviceRotation { get { return Read(_deviceRotation); } }
    InputFeatureUsage<Vector3>? _deviceVelocity;
    /// <summary>Rate of change in DevicePosition</summary>
    public Vector3 DeviceVelocity { get { return Read(_deviceVelocity); } }
    InputFeatureUsage<float>? _grip;
    /// <summary>How much is the grip trigger pressed. If the grip is a button
    /// (like on Vive Wands) then this changes value between 0 and 1.</summary>
    /// Use GripButton if you only need the boolean value
    public float Grip { get { return Read(_grip); } }
    InputFeatureUsage<bool>? _gripButton;
    /// <summary>Whether or not is the Grip considered pressed by the OpenXR
    /// runtime</summary>
    /// Use Grip to get more fine grained information about grip position.
    public bool GripButton { get { return Read(_gripButton); } }
    InputFeatureUsage<bool>? _isTracked;
    /// <summary>Whether this device is tracked. False when this device is
    /// connected but its position is unknown.</summary>
    public bool IsTracked { get { return Read(_isTracked); } }
    // Note: I merged Oculus Touch's MenuButton and Vive's Menu into MenuButton only
    InputFeatureUsage<bool>? _menuButton;
    /// <summary>Whether the app menu button is pressed.</summary>
    /// This returns value of "Menu" usage of Vive Wands because those are
    /// esentially the same thing with different name.
    public bool MenuButton { get { return Read(_menuButton); } }
    InputFeatureUsage<Vector3>? _pointerAngularVelocity;
    /// <summary>Rate of change of PointerRotation</summary>
    public Vector3 PointerAngularVelocity { get { return Read(_pointerAngularVelocity); } }
    InputFeatureUsage<Vector3>? _pointerPosition;
    /// <summary>Position of pointer reference frame of this controller relative
    /// to global tracking space</summary>
    /// There is usually a slight offset between device and pointer coordinates
    /// to account for different controllers feeling like they point in different
    /// directions then their body points to.
    /// <seealso cref="DevicePosition"/>
    public Vector3 PointerPosition { get { return Read(_pointerPosition); } }
    InputFeatureUsage<Quaternion>? _pointerRotation;
    /// <summary>Rotation of pointer reference frame of this controller relative
    /// to global tracking space</summary>
    /// <seealso cref="PointerPosition"/>
    public Quaternion PointerRotation { get { return Read(_pointerRotation); } }
    InputFeatureUsage<Vector3>? _pointerVelocity;
    /// <summary>Rate of change of PointerPosition</summary>
    public Vector3 PointerVelocity { get { return Read(_pointerVelocity); } }
    InputFeatureUsage<Vector2>? _primary2DAxis;
    /// <summary>Represents direction of primary joystick or where on the
    /// touchpad in case of Vive Wands is user pressing.</summary>
    /// Range is `<-1;1>x<-1;1>`
    public Vector2 Primary2DAxis { get { return Read(_primary2DAxis); } }
    InputFeatureUsage<bool>? _primary2DAxisClick;
    /// <summary>Whether or not is primary joystick/touchpad pressed in</summary>
    public bool Primary2DAxisClick { get { return Read(_primary2DAxisClick); } }
    InputFeatureUsage<bool>? _primary2DAxisTouch;
    /// <summary>Whether or not is user's finger touching primary joystick/touchpad</summary>
    public bool Primary2DAxisTouch { get { return Read(_primary2DAxisTouch); } }
    InputFeatureUsage<bool>? _primaryButton;
    /// <summary>Whether or not is primary button (A/X) on the controller pressed.</summary>
    /// This button is not present on Vive Wands.
    public bool PrimaryButton
    {
        get
        {
            if (_primaryButton == null) return false; // button is not there, it can't be pressed
            return Read(_primaryButton);
        }
    }
    /// <summary>Returns whether the PrimaryButton is available on this controller</summary>
    public bool PrimaryButtonAvailable { get { return _primaryButton != null; } }
    InputFeatureUsage<bool>? _primaryTouch;
    /// <summary>Returns whether the PrimaryButton is touched</summary>
    /// Not present on Vive Wand, might not be present even on controllers with
    /// primary buttons if their buttons are not capacitive.
    public bool PrimaryTouch
    {
        get
        {
            if (_primaryTouch == null) return false; // button is not there, it can't be touched
            return Read(_primaryTouch);
        }
    }
    /// <summary>Returns whether the PrimaryTouch is available on this controller</summary>
    public bool PrimaryTouchAvailable { get { return _primaryTouch != null; } }
    InputFeatureUsage<bool>? _secondaryButton;
    /// <summary>Whether or not is secondary button (B/Y) on the controller pressed.</summary>
    /// This button is not present on Vive Wands.
    public bool SecondaryButton
    {
        get
        {
            if (_secondaryButton == null) return false; // button is not there, it can't be pressed
            return Read(_secondaryButton);
        }
    }
    /// <summary>Returns whether the SecondaryButton is available on this controller</summary>
    public bool SecondaryButtonAvailable { get { return _secondaryButton != null; } }
    InputFeatureUsage<bool>? _secondaryTouch;
    /// <summary>Returns whether the SecondaryButton is touched</summary>
    /// Not present on Vive Wand, might not be present even on controllers with
    /// secondary buttons if their buttons are not capacitive.
    public bool SecondaryTouch
    {
        get
        {
            if (_secondaryTouch == null) return false; // button is not there, it can't be touched
            return Read(_secondaryTouch);
        }
    }
    /// <summary>Returns whether the SecondaryTouch is available on this controller</summary>
    public bool SecondaryTouchAvailable { get { return _secondaryTouch != null; } }
    InputFeatureUsage<bool>? _systemButton;
    /// <summary>Returns whether the SystemButton is pressed</summary>
    /// This only works when SystemButton is disabled in system settings,
    /// otherwise its intercepted by the runtime. Not available on Quest 2
    public bool SystemButton
    {
        get
        {
            if (_systemButton == null) return false; // button is not there, it can't be pressed
            return Read(_systemButton);
        }
    }
    /// <summary>Returns whether the SystemButton is available</summary>
    public bool SystemButtonAvailable { get { return _systemButton != null; } }
    InputFeatureUsage<uint>? _trackingState;
    /// <summary>Returns raw tracking state with more detail than IsTracked</summary>
    /// I did not research details of this field and therefore am unsure of how
    /// exactly it represents its values.
    public uint TrackingState { get { return Read(_trackingState); } }
    InputFeatureUsage<float>? _trigger;
    /// <summary>Returns how much is the trigger button pressed</summary>
    /// 0 means not at all, 1 means fully, 0.5 means half way
    ///
    /// If you do not need special settings and only need a boolean use TriggerButton instead
    public float Trigger { get { return Read(_trigger); } }
    InputFeatureUsage<bool>? _triggerButton;
    /// <summary>Returns whether trigger is fully pressed</summary>
    public bool TriggerButton { get { return Read(_triggerButton); } }
    InputFeatureUsage<bool>? _triggerTouch;
    /// <summary>Returns whether trigger button is touched</summary>
    /// On Vive Wands which do not have this in hardware this is emulated as
    /// true when the trigger is pressed a little bit.
    public bool TriggerTouch
    {
        get
        {
            // emulate using "slightly pressed trigger"
            if (_triggerTouch == null && _trigger != null) return Read(_trigger) > 0;
            return Read(_triggerTouch);
        }
    }
    /// <summary>Returns whether TriggerTouch is available in hardware</summary>
    public bool TriggerTouchAvailable { get { return _triggerTouch != null; } }
    // ADD_FEATURE step 1: add two fields above this line

    /// <summary>
    /// Constructor. I detect available Usages upon construction
    /// </summary>
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

    /// <summary>
    /// Internal function to initialize various private InputFeatureUsage fields
    /// in this class.
    /// </summary>
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

    /// <summary>
    /// Internal utility function to make it easier to implement value getters
    /// (they become one-liners). All other Read overloads have the same role.
    /// </summary>
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
