using System;
using UnityEngine;
using UnityEngine.XR;

/// <summary>
/// Main type representing XR Device. Can be synchronized via network or updated
/// from local IsblXRDevice.
/// </summary>
///
/// Only tested with Vive Wands and Oculus Quest 2 Touch controllers but there is
/// nothing that would prevent it working under other headsets provided that they
/// have the required buttons.
///
/// When button is missing from either Quest 2 or Vive Wands controller I provided
/// reasonable default (buttons always return false, because you can't press)
/// nonexistent button. Touch members return true if associated axis is pressed.
/// Position and velocity defaults to zero, rotation defaults to identity.
/// Otherwise it throws an exception upon reading the value.
///
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
public class IsblStaticXRDevice
{
    // ADDING_NEW_TYPE:
    // (search for ADDING_NEW_TYPE to see all relevant markers)
    // add new Type[] member
    Quaternion[] _dataQuaternion;
    Vector3[] _dataVector3;
    Vector2[] _dataVector2;
    float[] _dataFloat;
    bool[] _dataBool;
    uint[] _dataUint;

    public string Name = "";

    int _localUniqueId; // used for detecting device change, zero for remote devices

    // TODO: remove these
    public byte[] Data = new byte[0];
    public const int DataLength = 0;

    /// <summary>
    /// Denotes whether information stored outside of Data chaned
    /// </summary>
    ///
    /// True means that the data needs to be transmitted to the server.
    public bool DeviceInfoChanged;
    public bool IsLocal => _localUniqueId != 0;
    public InputDeviceCharacteristics Characteristics;

    // ADDING_NEW_USAGE:
    // (search for ADDING_NEW_USAGE to see all relevant markers)
    // create three new members here (private int _...Location, public Type, public bool ...Available)
    // please determine reasonable defaults if axis is not available
    int _deviceRotationLocation = -1;
    /// <summary>Rotation of this XR Device relative to tracking space</summary>
    public Quaternion DeviceRotation => _deviceRotationLocation >= 0 ? _dataQuaternion[_deviceRotationLocation] : Quaternion.identity;
    /// <summary>Returns whether the DeviceRotation is available</summary>
    public bool DeviceRotationAvailable => _deviceRotationLocation >= 0;

    int _pointerRotationLocation = -1;
    /// <summary>Rotation of pointer reference frame of this controller relative
    /// to global tracking space</summary>
    /// <seealso cref="PointerPosition"/>
    public Quaternion PointerRotation => _pointerRotationLocation >= 0 ? _dataQuaternion[_pointerRotationLocation] : Quaternion.identity;
    /// <summary>Returns whether the PointerRotation is available</summary>
    public bool PointerRotationAvailable => _pointerRotationLocation >= 0;

    int _deviceAngularVelocityLocation = -1;
    /// <summary>Rate of change in DeviceRotation</summary>
    public Vector3 DeviceAngularVelocity => _deviceAngularVelocityLocation >= 0 ? _dataVector3[_deviceAngularVelocityLocation] : Vector3.zero;
    /// <summary>Returns whether the DeviceAngularVelocity is available</summary>
    public bool DeviceAngularVelocityAvailable => _deviceAngularVelocityLocation >= 0;

    int _devicePositionLocation = -1;
    /// <summary>Position of this XR Device in tracking space</summary>
    public Vector3 DevicePosition => _devicePositionLocation >= 0 ? _dataVector3[_devicePositionLocation] : Vector3.zero;
    /// <summary>Returns whether the DevicePosition is available</summary>
    public bool DevicePositionAvailable => _devicePositionLocation >= 0;

    int _deviceVelocityLocation = -1;
    /// <summary>Rate of change in DevicePosition</summary>
    public Vector3 DeviceVelocity => _deviceVelocityLocation >= 0 ? _dataVector3[_deviceVelocityLocation] : Vector3.zero;
    /// <summary>Returns whether the DeviceVelocity is available</summary>
    public bool DeviceVelocityAvailable => _deviceVelocityLocation >= 0;

    int _pointerAngularVelocityLocation = -1;
    /// <summary>Rate of change of PointerRotation</summary>
    public Vector3 PointerAngularVelocity => _pointerAngularVelocityLocation >= 0 ? _dataVector3[_pointerAngularVelocityLocation] : Vector3.zero;
    /// <summary>Returns whether the PointerAngularVelocity is available</summary>
    public bool PointerAngularVelocityAvailable => _pointerAngularVelocityLocation >= 0;

    int _pointerPositionLocation = -1;
    /// <summary>Position of pointer reference frame of this controller relative
    /// to global tracking space</summary>
    /// There is usually a slight offset between device and pointer coordinates
    /// to account for different controllers feeling like they point in different
    /// directions then their body points to.
    /// <seealso cref="DevicePosition"/>
    public Vector3 PointerPosition => _pointerPositionLocation >= 0 ? _dataVector3[_pointerPositionLocation] : Vector3.zero;
    /// <summary>Returns whether the PointerPosition is available</summary>
    public bool PointerPositionAvailable => _pointerPositionLocation >= 0;

    int _pointerVelocityLocation = -1;
    /// <summary>Rate of change of PointerPosition</summary>
    public Vector3 PointerVelocity => _pointerVelocityLocation >= 0 ? _dataVector3[_pointerVelocityLocation] : Vector3.zero;
    /// <summary>Returns whether the PointerVelocity is available</summary>
    public bool PointerVelocityAvailable => _pointerVelocityLocation >= 0;

    int _primary2DAxisLocation = -1;
    /// <summary>Represents direction of primary joystick or where on the
    /// touchpad in case of Vive Wands is user pressing.</summary>
    /// Range is `<-1;1>x<-1;1>`
    public Vector2 Primary2DAxis => _primary2DAxisLocation >= 0
        ? _dataVector2[_primary2DAxisLocation]
        : Vector2.zero;
    /// <summary>Returns whether the Primary2DAxis is available</summary>
    public bool Primary2DAxisAvailable => _primary2DAxisLocation >= 0;

    int _gripLocation = -1;
    /// <summary>How much is the grip trigger pressed. If the grip is a button
    /// (like on Vive Wands) then this changes value between 0 and 1.</summary>
    /// Use GripButton if you only need the boolean value
    public float Grip => _gripLocation >= 0 ? _dataFloat[_gripLocation] : 0;
    /// <summary>Returns whether the Grip is available</summary>
    public bool GripAvailable => _gripLocation >= 0;

    int _triggerLocation = -1;
    /// <summary>Returns how much is the trigger button pressed</summary>
    /// 0 means not at all, 1 means fully, 0.5 means half way
    ///
    /// If you do not need special settings and only need a boolean use TriggerButton instead
    public float Trigger => _triggerLocation >= 0 ? _dataFloat[_triggerLocation] : 0;
    /// <summary>Returns whether the Trigger is available</summary>
    public bool TriggerAvailable => _triggerLocation >= 0;

    int _trackingStateLocation = -1;
    /// <summary>Returns raw tracking state with more detail than IsTracked</summary>
    /// I did not research details of this field and therefore am unsure of how
    /// exactly it represents its values.
    public uint TrackingState => _trackingStateLocation >= 0 ? _dataUint[_trackingStateLocation] : 0;
    /// <summary>Returns whether the TrackingState is available</summary>
    public bool TrackingStateAvailable => _trackingStateLocation >= 0;

    int _gripButtonLocation = -1;
    /// <summary>Whether or not is the Grip considered pressed by the OpenXR
    /// runtime</summary>
    /// Use Grip to get more fine grained information about grip position.
    public bool GripButton => _gripButtonLocation >= 0 && _dataBool[_gripButtonLocation];
    /// <summary>Returns whether the GripButton is available</summary>
    public bool GripButtonAvailable => _gripButtonLocation >= 0;

    int _isTrackedLocation = -1;
    /// <summary>Whether this device is tracked. False when this device is
    /// connected but its position is unknown.</summary>
    public bool IsTracked => _isTrackedLocation >= 0 && _dataBool[_isTrackedLocation];
    /// <summary>Returns whether the IsTracked is available</summary>
    public bool IsTrackedAvailable => _isTrackedLocation >= 0;

    int _menuButtonLocation = -1;
    /// <summary>Whether the app menu button is pressed.</summary>
    /// This returns value of "Menu" usage of Vive Wands because those are
    /// esentially the same thing with different name.
    public bool MenuButton => _menuButtonLocation >= 0 && _dataBool[_menuButtonLocation];
    /// <summary>Returns whether the MenuButton is available</summary>
    public bool MenuButtonAvailable => _menuButtonLocation >= 0;

    int _primary2DAxisClickLocation = -1;
    /// <summary>Whether or not is primary joystick/touchpad pressed in</summary>
    public bool Primary2DAxisClick => _primary2DAxisClickLocation >= 0 && _dataBool[_primary2DAxisClickLocation];
    /// <summary>Returns whether the Primary2DAxisClick is available</summary>
    public bool Primary2DAxisClickAvailable => _primary2DAxisClickLocation >= 0;

    int _primary2DAxisTouchLocation = -1;
    /// <summary>Whether or not is user's finger touching primary joystick/touchpad</summary>
    public bool Primary2DAxisTouch => _primary2DAxisTouchLocation >= 0 ? _dataBool[_primary2DAxisTouchLocation] : Primary2DAxisClick;
    /// <summary>Returns whether the Primary2DAxisTouch is available</summary>
    public bool Primary2DAxisTouchAvailable => _primary2DAxisTouchLocation >= 0;

    int _primaryButtonLocation = -1;
    /// <summary>Whether or not is primary button (A/X) on the controller pressed.</summary>
    /// This button is not present on Vive Wands.
    public bool PrimaryButton => _primaryButtonLocation >= 0 && _dataBool[_primaryButtonLocation];
    /// <summary>Returns whether the PrimaryButton is available on this controller</summary>
    public bool PrimaryButtonAvailable => _primaryButtonLocation >= 0;

    int _primaryTouchLocation = -1;
    /// <summary>Returns whether the PrimaryButton is touched</summary>
    /// Not present on Vive Wand, might not be present even on controllers with
    /// primary buttons if their buttons are not capacitive.
    public bool PrimaryTouch => _primaryTouchLocation >= 0 ? _dataBool[_primaryTouchLocation] : PrimaryButton;
    /// <summary>Returns whether the PrimaryTouch is available on this controller</summary>
    public bool PrimaryTouchAvailable => _primaryTouchLocation >= 0;

    int _secondaryButtonLocation = -1;
    /// <summary>Whether or not is secondary button (B/Y) on the controller pressed.</summary>
    /// This button is not present on Vive Wands.
    public bool SecondaryButton => _secondaryButtonLocation >= 0 && _dataBool[_secondaryButtonLocation];
    /// <summary>Returns whether the SecondaryButton is available</summary>
    public bool SecondaryButtonAvailable => _secondaryButtonLocation >= 0;

    int _secondaryTouchLocation = -1;
    /// <summary>Returns whether the SecondaryButton is touched</summary>
    /// Not present on Vive Wand, might not be present even on controllers with
    /// secondary buttons if their buttons are not capacitive.
    public bool SecondaryTouch => _secondaryTouchLocation >= 0 ? _dataBool[_secondaryTouchLocation] : SecondaryButton;
    /// <summary>Returns whether the SecondaryTouch is available</summary>
    public bool SecondaryTouchAvailable => _secondaryTouchLocation >= 0;

    int _systemButtonLocation = -1;
    /// <summary>Returns whether the SystemButton is pressed</summary>
    /// This only works when SystemButton is disabled in system settings,
    /// otherwise its intercepted by the runtime. Not available on Quest 2
    public bool SystemButton => _systemButtonLocation >= 0 && _dataBool[_systemButtonLocation];
    /// <summary>Returns whether the SystemButton is available</summary>
    public bool SystemButtonAvailable => _systemButtonLocation >= 0;

    int _triggerButtonLocation = -1;
    /// <summary>Returns whether trigger is pressed enough</summary>
    public bool TriggerButton => _triggerButtonLocation >= 0 && _dataBool[_triggerButtonLocation];
    /// <summary>Returns whether the TriggerButton is available</summary>
    public bool TriggerButtonAvailable => _triggerButtonLocation >= 0;

    int _triggerTouchLocation = -1;
    /// <summary>Returns whether trigger button is touched</summary>
    /// On Vive Wands which do not have this in hardware this is emulated as
    /// true when the trigger is pressed a little bit.
    public bool TriggerTouch => _triggerTouchLocation >= 0
        ? _dataBool[_triggerTouchLocation]
        : TriggerAvailable && Trigger > 0;
    /// <summary>Returns whether TriggerTouch is available in hardware</summary>
    public bool TriggerTouchAvailable => _triggerTouchLocation >= 0;

    /// <summary>
    /// Reads data from device to update internal state
    /// </summary>
    public void UpdateFromDevice(IsblXRDevice device)
    {
        if (device == null)
        {
            if (_localUniqueId != 0)
            {
                Name = "";
                Characteristics = 0;
                _localUniqueId = 0;
                DeviceInfoChanged = true;

                // ADDING_NEW_TYPE:
                // add if branch here
                _dataQuaternion = null;
                _dataVector3 = null;
                _dataVector2 = null;
                _dataFloat = null;
                _dataBool = null;
                _dataUint = null;

                // ADDING_NEW_USAGE:
                // add line here
                _deviceRotationLocation = -1;
                _pointerRotationLocation = -1;
                _deviceAngularVelocityLocation = -1;
                _devicePositionLocation = -1;
                _deviceVelocityLocation = -1;
                _pointerAngularVelocityLocation = -1;
                _pointerPositionLocation = -1;
                _pointerVelocityLocation = -1;
                _primary2DAxisLocation = -1;
                _gripLocation = -1;
                _triggerLocation = -1;
                _gripButtonLocation = -1;
                _isTrackedLocation = -1;
                _menuButtonLocation = -1;
                _primary2DAxisClickLocation = -1;
                _primary2DAxisTouchLocation = -1;
                _primaryButtonLocation = -1;
                _primaryTouchLocation = -1;
                _secondaryButtonLocation = -1;
                _secondaryTouchLocation = -1;
                _systemButtonLocation = -1;
                _triggerButtonLocation = -1;
                _triggerTouchLocation = -1;
                _trackingStateLocation = -1;
            }
            return;
        }

        if (_localUniqueId != device.LocallyUniqueId)
        {
            _localUniqueId = device.LocallyUniqueId;
            Characteristics = device.Characteristics;
            Name = device.Name;
            DeviceInfoChanged = true;

            // ADDING_NEW_USAGE:
            // add if branch to relevant for loop
            for (var i = 0; i < device.Quaternion.Length; ++i)
            {
                var name = device.Quaternion[i].name;
                if (name == "DeviceRotation") _deviceRotationLocation = i;
                else if (name == "PointerRotation") _pointerRotationLocation = i;
                else Debug.Log($"Unknown usage device of type Quaternion with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Vector3.Length; ++i)
            {
                var name = device.Vector3[i].name;
                if (name == "DeviceAngularVelocity") _deviceAngularVelocityLocation = i;
                else if (name == "DevicePosition") _devicePositionLocation = i;
                else if (name == "DeviceVelocity") _deviceVelocityLocation = i;
                else if (name == "PointerAngularVelocity") _pointerAngularVelocityLocation = i;
                else if (name == "PointerPosition") _pointerPositionLocation = i;
                else if (name == "PointerVelocity") _pointerVelocityLocation = i;
                else Debug.Log($"Unknown usage device of type Vector3 with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Vector2.Length; ++i)
            {
                var name = device.Vector2[i].name;
                if (name == "Primary2DAxis") _primary2DAxisLocation = i;
                else Debug.Log($"Unknown usage device of type Vector2 with name {name} on device {device.Name}");
            }
            for (var i = 0; i < device.Float.Length; ++i)
            {
                var name = device.Float[i].name;
                if (name == "Grip") _gripLocation = i;
                else if (name == "Trigger") _triggerLocation = i;
                else Debug.Log($"Unknown usage device of type float with name {name} on device {device.Name}");
            }
            for (var i = 0; i < device.Bool.Length; ++i)
            {
                var name = device.Bool[i].name;
                if (name == "GripButton") _gripButtonLocation = i;
                else if (name == "IsTracked") _isTrackedLocation = i;
                // Note: I merged Oculus Touch's MenuButton and Vive's Menu into MenuButton only
                else if (name == "Menu" || name == "MenuButton") _menuButtonLocation = i;
                else if (name == "Primary2DAxisClick") _primary2DAxisClickLocation = i;
                else if (name == "Primary2DAxisTouch") _primary2DAxisTouchLocation = i;
                else if (name == "PrimaryButton") _primaryButtonLocation = i;
                else if (name == "PrimaryTouch") _primaryTouchLocation = i;
                else if (name == "SecondaryButton") _secondaryButtonLocation = i;
                else if (name == "SecondaryTouch") _secondaryTouchLocation = i;
                else if (name == "SystemButton") _systemButtonLocation = i;
                else if (name == "TriggerButton") _triggerButtonLocation = i;
                else if (name == "TriggerTouch") _triggerTouchLocation = i;
                else Debug.Log($"Unknown usage device of type bool with name {name} on device {device.Name}");
            }
            for (var i = 0; i < device.Uint.Length; ++i)
            {
                var name = device.Uint[i].name;
                if (name == "TrackingState") _trackingStateLocation = i;
                else Debug.Log($"Unknown usage device of type uint with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Bone.Length; ++i)
            {
                var name = device.Bone[i].name;
                Debug.Log($"Unknown usage device of type Bone with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Hand.Length; ++i)
            {
                var name = device.Hand[i].name;
                Debug.Log($"Unknown usage device of type Hand with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.InputTrackingState.Length; ++i)
            {
                var name = device.InputTrackingState[i].name;
                Debug.Log($"Unknown usage device of type InputTrackingState with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.ByteArray.Length; ++i)
            {
                var name = device.ByteArray[i].name;
                Debug.Log($"Unknown usage device of type Byte with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Hand.Length; ++i)
            {
                var name = device.Eyes[i].name;
                Debug.Log($"Unknown usage device of type Eyes with name {name} on device {device.Name}");
            }

            // ADDING_NEW_TYPE:
            // add if branch here
            if (_dataQuaternion == null || _dataQuaternion.Length != device.Quaternion.Length) _dataQuaternion = new Quaternion[device.Quaternion.Length];
            if (_dataVector3 == null || _dataVector3.Length != device.Vector3.Length) _dataVector3 = new Vector3[device.Vector3.Length];
            if (_dataVector2 == null || _dataVector2.Length != device.Vector2.Length) _dataVector2 = new Vector2[device.Vector2.Length];
            if (_dataFloat == null || _dataFloat.Length != device.Float.Length) _dataFloat = new float[device.Float.Length];
            if (_dataBool == null || _dataBool.Length != device.Bool.Length) _dataBool = new bool[device.Bool.Length];
            if (_dataUint == null || _dataUint.Length != device.Uint.Length) _dataUint = new uint[device.Uint.Length];
        }

        // ADDING_NEW_TYPE:
        // add for loop here
        for (var i = 0; i < device.Quaternion.Length; ++i) device.Device.TryGetFeatureValue(device.Quaternion[i], out _dataQuaternion[i]);
        for (var i = 0; i < device.Vector3.Length; ++i) device.Device.TryGetFeatureValue(device.Vector3[i], out _dataVector3[i]);
        for (var i = 0; i < device.Vector2.Length; ++i) device.Device.TryGetFeatureValue(device.Vector2[i], out _dataVector2[i]);
        for (var i = 0; i < device.Float.Length; ++i) device.Device.TryGetFeatureValue(device.Float[i], out _dataFloat[i]);
        for (var i = 0; i < device.Bool.Length; ++i) device.Device.TryGetFeatureValue(device.Bool[i], out _dataBool[i]);
        for (var i = 0; i < device.Uint.Length; ++i) device.Device.TryGetFeatureValue(device.Uint[i], out _dataUint[i]);
    }
}
