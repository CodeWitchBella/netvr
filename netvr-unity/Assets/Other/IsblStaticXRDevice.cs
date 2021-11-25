using System;
using System.IO;
using Newtonsoft.Json.Linq;
using Newtonsoft.Json.Serialization;
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
    #region members (outside this region go only getters and methods, ie. no data)
    // ADDING_NEW_TYPE:
    // (search for ADDING_NEW_TYPE to see all relevant markers)
    // add new Type[] member
    Quaternion[] _dataQuaternion;
    Vector3[] _dataVector3;
    Vector2[] _dataVector2;
    float[] _dataFloat;
    bool[] _dataBool;
    uint[] _dataUint;

    public string Name { get; private set; }
    int _localUniqueId; // used for detecting device change, zero for remote devices
    public InputDeviceCharacteristics Characteristics { get; private set; }
    Locations _locations = new();
    #endregion

    /// <summary>
    /// Denotes whether information stored outside of Data chaned
    /// </summary>
    ///
    /// True means that the data needs to be transmitted to the server.
    public bool DeviceInfoChanged;

    // ADDING_NEW_TYPE:
    // Increment this by one
    const int TypeCount = 6;

    #region (De)Serialization
    /// <summary>
    /// Count number of bytes required to 7bit encode the number
    /// </summary>
    static int CountSizeBytes(int count, int perElement)
    {
        int r = perElement * count + 1;
        while ((count >>= 7) != 0) r++;
        return r;
    }

    static void Write7BitEncodedInt(BinaryWriter writer, int i)
    {
        do
        {
            var next = i >> 7;
            writer.Write((byte)((next != 0 ? 0x80 : 0) | i));
            i = next;
        } while (i != 0);
    }

    // Copied from: https://github.com/dotnet/runtime/issues/24473#issuecomment-450755980
    static int Read7BitEncodedInt(BinaryReader reader)
    {
        sbyte b;
        int r = -7, v = 0;
        do
            v |= ((b = reader.ReadSByte()) & 0x7F) << (r += 7);
        while (b < 0);
        return v;
    }

    public int CalculateSerializationSize()
    {
        if (_dataQuaternion == null) return TypeCount;
        // ADDING_NEW_TYPE:
        // add line for calculating serialization size here
        return CountSizeBytes(_dataQuaternion.Length, 4 * 3)
            + CountSizeBytes(_dataVector3.Length, 4 * 3)
            + CountSizeBytes(_dataVector2.Length, 4 * 2)
            + CountSizeBytes(_dataFloat.Length, 4)
            + CountSizeBytes(_dataBool.Length, 1)
            + CountSizeBytes(_dataUint.Length, 4);
    }

    class SerializationSection : IDisposable
    {
        MemoryStream _stream;
        int _initialPosition;
        int _expectedSize;
        public SerializationSection(MemoryStream stream, int expectedSize)
        {
            _stream = stream;
            _initialPosition = (int)stream.Position;
            _expectedSize = expectedSize;
        }
        public void Dispose()
        {
            if (_stream.Position - _initialPosition != _expectedSize)
            {
                Debug.LogWarning($"Expected section to write {_expectedSize} bytes. Got {_stream.Position - _initialPosition} bytes instead.");
            }
        }
    }

    public int DeSerializeData(byte[] source, int offset)
    {
        MemoryStream stream = new(source);
        stream.Position = offset;
        BinaryReader reader = new(stream);

        using (new SerializationSection(stream, CountSizeBytes(_dataQuaternion.Length, 4 * 3)))
        {
            var len = Read7BitEncodedInt(reader);
            if (_dataQuaternion?.Length == len)
                for (int i = 0; i < len; ++i) _dataQuaternion[i] = Quaternion.Euler(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            else
                for (int i = 0; i < len; ++i) Quaternion.Euler(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataVector3.Length, 4 * 3)))
        {
            var len = Read7BitEncodedInt(reader);
            if (_dataVector3?.Length == len)
                for (int i = 0; i < len; ++i) _dataVector3[i] = new Vector3(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            else
                for (int i = 0; i < len; ++i) new Vector3(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataVector2.Length, 4 * 3)))
        {
            var len = Read7BitEncodedInt(reader);
            if (_dataVector2?.Length == len)
                for (int i = 0; i < len; ++i) _dataVector2[i] = new Vector2(reader.ReadSingle(), reader.ReadSingle());
            else
                for (int i = 0; i < len; ++i) new Vector2(reader.ReadSingle(), reader.ReadSingle());
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataFloat.Length, 4 * 3)))
        {
            var len = Read7BitEncodedInt(reader);
            if (_dataFloat?.Length == len)
                for (int i = 0; i < len; ++i) _dataFloat[i] = reader.ReadSingle();
            else
                for (int i = 0; i < len; ++i) reader.ReadSingle();
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataBool.Length, 4 * 3)))
        {
            var len = Read7BitEncodedInt(reader);
            if (_dataBool?.Length == len)
                for (int i = 0; i < len; ++i) _dataBool[i] = reader.ReadBoolean();
            else
                for (int i = 0; i < len; ++i) reader.ReadBoolean();
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataUint.Length, 4 * 3)))
        {
            var len = Read7BitEncodedInt(reader);
            if (_dataUint?.Length == len)
                for (int i = 0; i < len; ++i) _dataUint[i] = reader.ReadUInt32();
            else
                for (int i = 0; i < len; ++i) reader.ReadUInt32();
        }

        // ADDING_NEW_TYPE:
        // Add deserialization code above this comment

        return (int)stream.Position - offset;
    }

    public int SerializeData(byte[] target, int offset)
    {
        MemoryStream stream = new(target);
        stream.Position = offset;
        BinaryWriter writer = new(stream);
        if (_dataQuaternion == null)
        {
            for (int i = 0; i < TypeCount; ++i)
                Write7BitEncodedInt(writer, 0);
            return TypeCount;
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataQuaternion.Length, 4 * 3)))
        {
            Write7BitEncodedInt(writer, _dataQuaternion.Length);
            foreach (var el in _dataQuaternion)
            {
                writer.Write(el.eulerAngles.x);
                writer.Write(el.eulerAngles.y);
                writer.Write(el.eulerAngles.z);
            }
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataVector3.Length, 4 * 3)))
        {
            Write7BitEncodedInt(writer, _dataVector3.Length);
            foreach (var el in _dataVector3)
            {
                writer.Write(el.x);
                writer.Write(el.y);
                writer.Write(el.z);
            }
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataVector2.Length, 4 * 2)))
        {
            Write7BitEncodedInt(writer, _dataVector2.Length);
            foreach (var el in _dataVector2)
            {
                writer.Write(el.x);
                writer.Write(el.y);
            }
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataFloat.Length, 4)))
        {
            Write7BitEncodedInt(writer, _dataFloat.Length);
            foreach (var el in _dataFloat) writer.Write(el);
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataBool.Length, 1)))
        {
            Write7BitEncodedInt(writer, _dataBool.Length);
            foreach (var el in _dataBool) writer.Write(el);
        }

        using (new SerializationSection(stream, CountSizeBytes(_dataUint.Length, 4)))
        {
            Write7BitEncodedInt(writer, _dataUint.Length);
            foreach (var el in _dataUint) writer.Write(el);
        }

        // ADDING_NEW_TYPE:
        // Add serialization code above this comment

        return (int)stream.Position - offset;
    }

    public JObject SerializeConfiguration()
    {
        JArray characteristics = new();
        void Check(InputDeviceCharacteristics reference, string text)
        {
            if ((Characteristics & reference) != 0)
                characteristics.Add(text);
        }
        Check(InputDeviceCharacteristics.HeadMounted, "HeadMounted");
        Check(InputDeviceCharacteristics.Camera, "Camera");
        Check(InputDeviceCharacteristics.HeldInHand, "HeldInHand");
        Check(InputDeviceCharacteristics.HandTracking, "HandTracking");
        Check(InputDeviceCharacteristics.EyeTracking, "EyeTracking");
        Check(InputDeviceCharacteristics.TrackedDevice, "TrackedDevice");
        Check(InputDeviceCharacteristics.Controller, "Controller");
        Check(InputDeviceCharacteristics.TrackingReference, "TrackingReference");
        Check(InputDeviceCharacteristics.Left, "Left");
        Check(InputDeviceCharacteristics.Right, "Right");
        Check(InputDeviceCharacteristics.Simulated6DOF, "Simulated6DOF");

        return JObject.FromObject(new
        {
            locations = JObject.FromObject(_locations, new Newtonsoft.Json.JsonSerializer()
            { ContractResolver = new CamelCasePropertyNamesContractResolver() }),
            name = Name,
            characteristics,
            lengths = new
            {
                quaternion = _dataQuaternion?.Length ?? 0,
                vector3 = _dataVector3?.Length ?? 0,
                vector2 = _dataVector2?.Length ?? 0,
                @float = _dataFloat?.Length ?? 0,
                @bool = _dataBool?.Length ?? 0,
                @uint = _dataUint?.Length ?? 0,
                // ADDING_NEW_TYPE:
                // Add line here
            }
        });
    }

    public void DeSerializeConfiguration(JObject message)
    {
        Characteristics = 0;
        foreach (var c in message.Value<JArray>("characteristics"))
        {
            Characteristics |= (string)c switch
            {
                "HeadMounted" => InputDeviceCharacteristics.HeadMounted,
                "Camera" => InputDeviceCharacteristics.Camera,
                "HeldInHand" => InputDeviceCharacteristics.HeldInHand,
                "HandTracking" => InputDeviceCharacteristics.HandTracking,
                "EyeTracking" => InputDeviceCharacteristics.EyeTracking,
                "TrackedDevice" => InputDeviceCharacteristics.TrackedDevice,
                "Controller" => InputDeviceCharacteristics.Controller,
                "TrackingReference" => InputDeviceCharacteristics.TrackingReference,
                "Left" => InputDeviceCharacteristics.Left,
                "Right" => InputDeviceCharacteristics.Right,
                "Simulated6DOF" => InputDeviceCharacteristics.Simulated6DOF,
                _ => 0,
            };
        }
        _locations = message.Value<Locations>("locations");
        Name = message.Value<string>("name");
        var lengths = message.Value<JObject>("lengths");
        _dataQuaternion = new Quaternion[lengths.Value<int>("quaternion")];
        _dataVector3 = new Vector3[lengths.Value<int>("vector3")];
        _dataVector2 = new Vector2[lengths.Value<int>("vector2")];
        _dataFloat = new float[lengths.Value<int>("float")];
        _dataBool = new bool[lengths.Value<int>("bool")];
        _dataUint = new uint[lengths.Value<int>("uint")];
        // ADDING_NEW_TYPE:
        // Add line here
    }
    #endregion

    public bool IsLocal => _localUniqueId != 0;

    class Locations
    {
        // ADDING_NEW_USAGE:
        // (search for ADDING_NEW_USAGE to see all relevant markers)
        // add new member here
        public int DeviceRotation = -1;
        public int PointerRotation = -1;
        public int DeviceAngularVelocity = -1;
        public int DevicePosition = -1;
        public int DeviceVelocity = -1;
        public int PointerAngularVelocity = -1;
        public int PointerPosition = -1;
        public int PointerVelocity = -1;
        public int Primary2DAxis = -1;
        public int Grip = -1;
        public int Trigger = -1;
        public int TrackingState = -1;
        public int GripButton = -1;
        public int IsTracked = -1;
        public int MenuButton = -1;
        public int Primary2DAxisClick = -1;
        public int Primary2DAxisTouch = -1;
        public int PrimaryButton = -1;
        public int PrimaryTouch = -1;
        public int SecondaryButton = -1;
        public int SecondaryTouch = -1;
        public int SystemButton = -1;
        public int TriggerButton = -1;
        public int TriggerTouch = -1;
    };

    // ADDING_NEW_USAGE:
    // create two new getters here (public Type, public bool ...Available)
    // please determine reasonable defaults if axis is not available
    /// <summary>Rotation of this XR Device relative to tracking space</summary>
    public Quaternion DeviceRotation => _locations.DeviceRotation >= 0 ? _dataQuaternion[_locations.DeviceRotation] : Quaternion.identity;
    /// <summary>Returns whether the DeviceRotation is available</summary>
    public bool DeviceRotationAvailable => _locations.DeviceRotation >= 0;

    /// <summary>Rotation of pointer reference frame of this controller relative
    /// to global tracking space</summary>
    /// <seealso cref="PointerPosition"/>
    public Quaternion PointerRotation => _locations.PointerRotation >= 0 ? _dataQuaternion[_locations.PointerRotation] : Quaternion.identity;
    /// <summary>Returns whether the PointerRotation is available</summary>
    public bool PointerRotationAvailable => _locations.PointerRotation >= 0;

    /// <summary>Rate of change in DeviceRotation</summary>
    public Vector3 DeviceAngularVelocity => _locations.DeviceAngularVelocity >= 0 ? _dataVector3[_locations.DeviceAngularVelocity] : Vector3.zero;
    /// <summary>Returns whether the DeviceAngularVelocity is available</summary>
    public bool DeviceAngularVelocityAvailable => _locations.DeviceAngularVelocity >= 0;

    /// <summary>Position of this XR Device in tracking space</summary>
    public Vector3 DevicePosition => _locations.DevicePosition >= 0 ? _dataVector3[_locations.DevicePosition] : Vector3.zero;
    /// <summary>Returns whether the DevicePosition is available</summary>
    public bool DevicePositionAvailable => _locations.DevicePosition >= 0;

    /// <summary>Rate of change in DevicePosition</summary>
    public Vector3 DeviceVelocity => _locations.DeviceVelocity >= 0 ? _dataVector3[_locations.DeviceVelocity] : Vector3.zero;
    /// <summary>Returns whether the DeviceVelocity is available</summary>
    public bool DeviceVelocityAvailable => _locations.DeviceVelocity >= 0;

    /// <summary>Rate of change of PointerRotation</summary>
    public Vector3 PointerAngularVelocity => _locations.PointerAngularVelocity >= 0 ? _dataVector3[_locations.PointerAngularVelocity] : Vector3.zero;
    /// <summary>Returns whether the PointerAngularVelocity is available</summary>
    public bool PointerAngularVelocityAvailable => _locations.PointerAngularVelocity >= 0;

    /// <summary>Position of pointer reference frame of this controller relative
    /// to global tracking space</summary>
    /// There is usually a slight offset between device and pointer coordinates
    /// to account for different controllers feeling like they point in different
    /// directions then their body points to.
    /// <seealso cref="DevicePosition"/>
    public Vector3 PointerPosition => _locations.PointerPosition >= 0 ? _dataVector3[_locations.PointerPosition] : Vector3.zero;
    /// <summary>Returns whether the PointerPosition is available</summary>
    public bool PointerPositionAvailable => _locations.PointerPosition >= 0;

    /// <summary>Rate of change of PointerPosition</summary>
    public Vector3 PointerVelocity => _locations.PointerVelocity >= 0 ? _dataVector3[_locations.PointerVelocity] : Vector3.zero;
    /// <summary>Returns whether the PointerVelocity is available</summary>
    public bool PointerVelocityAvailable => _locations.PointerVelocity >= 0;

    /// <summary>Represents direction of primary joystick or where on the
    /// touchpad in case of Vive Wands is user pressing.</summary>
    /// Range is `<-1;1>x<-1;1>`
    public Vector2 Primary2DAxis => _locations.Primary2DAxis >= 0
        ? _dataVector2[_locations.Primary2DAxis]
        : Vector2.zero;
    /// <summary>Returns whether the Primary2DAxis is available</summary>
    public bool Primary2DAxisAvailable => _locations.Primary2DAxis >= 0;

    /// <summary>How much is the grip trigger pressed. If the grip is a button
    /// (like on Vive Wands) then this changes value between 0 and 1.</summary>
    /// Use GripButton if you only need the boolean value
    public float Grip => _locations.Grip >= 0 ? _dataFloat[_locations.Grip] : 0;
    /// <summary>Returns whether the Grip is available</summary>
    public bool GripAvailable => _locations.Grip >= 0;

    /// <summary>Returns how much is the trigger button pressed</summary>
    /// 0 means not at all, 1 means fully, 0.5 means half way
    ///
    /// If you do not need special settings and only need a boolean use TriggerButton instead
    public float Trigger => _locations.Trigger >= 0 ? _dataFloat[_locations.Trigger] : 0;
    /// <summary>Returns whether the Trigger is available</summary>
    public bool TriggerAvailable => _locations.Trigger >= 0;

    /// <summary>Returns raw tracking state with more detail than IsTracked</summary>
    /// I did not research details of this field and therefore am unsure of how
    /// exactly it represents its values.
    public uint TrackingState => _locations.TrackingState >= 0 ? _dataUint[_locations.TrackingState] : 0;
    /// <summary>Returns whether the TrackingState is available</summary>
    public bool TrackingStateAvailable => _locations.TrackingState >= 0;

    /// <summary>Whether or not is the Grip considered pressed by the OpenXR
    /// runtime</summary>
    /// Use Grip to get more fine grained information about grip position.
    public bool GripButton => _locations.GripButton >= 0 && _dataBool[_locations.GripButton];
    /// <summary>Returns whether the GripButton is available</summary>
    public bool GripButtonAvailable => _locations.GripButton >= 0;

    /// <summary>Whether this device is tracked. False when this device is
    /// connected but its position is unknown.</summary>
    public bool IsTracked => _locations.IsTracked >= 0 && _dataBool[_locations.IsTracked];
    /// <summary>Returns whether the IsTracked is available</summary>
    public bool IsTrackedAvailable => _locations.IsTracked >= 0;

    /// <summary>Whether the app menu button is pressed.</summary>
    /// This returns value of "Menu" usage of Vive Wands because those are
    /// esentially the same thing with different name.
    public bool MenuButton => _locations.MenuButton >= 0 && _dataBool[_locations.MenuButton];
    /// <summary>Returns whether the MenuButton is available</summary>
    public bool MenuButtonAvailable => _locations.MenuButton >= 0;

    /// <summary>Whether or not is primary joystick/touchpad pressed in</summary>
    public bool Primary2DAxisClick => _locations.Primary2DAxisClick >= 0 && _dataBool[_locations.Primary2DAxisClick];
    /// <summary>Returns whether the Primary2DAxisClick is available</summary>
    public bool Primary2DAxisClickAvailable => _locations.Primary2DAxisClick >= 0;

    /// <summary>Whether or not is user's finger touching primary joystick/touchpad</summary>
    public bool Primary2DAxisTouch => _locations.Primary2DAxisTouch >= 0 ? _dataBool[_locations.Primary2DAxisTouch] : Primary2DAxisClick;
    /// <summary>Returns whether the Primary2DAxisTouch is available</summary>
    public bool Primary2DAxisTouchAvailable => _locations.Primary2DAxisTouch >= 0;

    /// <summary>Whether or not is primary button (A/X) on the controller pressed.</summary>
    /// This button is not present on Vive Wands.
    public bool PrimaryButton => _locations.PrimaryButton >= 0 && _dataBool[_locations.PrimaryButton];
    /// <summary>Returns whether the PrimaryButton is available on this controller</summary>
    public bool PrimaryButtonAvailable => _locations.PrimaryButton >= 0;

    /// <summary>Returns whether the PrimaryButton is touched</summary>
    /// Not present on Vive Wand, might not be present even on controllers with
    /// primary buttons if their buttons are not capacitive.
    public bool PrimaryTouch => _locations.PrimaryTouch >= 0 ? _dataBool[_locations.PrimaryTouch] : PrimaryButton;
    /// <summary>Returns whether the PrimaryTouch is available on this controller</summary>
    public bool PrimaryTouchAvailable => _locations.PrimaryTouch >= 0;

    /// <summary>Whether or not is secondary button (B/Y) on the controller pressed.</summary>
    /// This button is not present on Vive Wands.
    public bool SecondaryButton => _locations.SecondaryButton >= 0 && _dataBool[_locations.SecondaryButton];
    /// <summary>Returns whether the SecondaryButton is available</summary>
    public bool SecondaryButtonAvailable => _locations.SecondaryButton >= 0;

    /// <summary>Returns whether the SecondaryButton is touched</summary>
    /// Not present on Vive Wand, might not be present even on controllers with
    /// secondary buttons if their buttons are not capacitive.
    public bool SecondaryTouch => _locations.SecondaryTouch >= 0 ? _dataBool[_locations.SecondaryTouch] : SecondaryButton;
    /// <summary>Returns whether the SecondaryTouch is available</summary>
    public bool SecondaryTouchAvailable => _locations.SecondaryTouch >= 0;

    /// <summary>Returns whether the SystemButton is pressed</summary>
    /// This only works when SystemButton is disabled in system settings,
    /// otherwise its intercepted by the runtime. Not available on Quest 2
    public bool SystemButton => _locations.SystemButton >= 0 && _dataBool[_locations.SystemButton];
    /// <summary>Returns whether the SystemButton is available</summary>
    public bool SystemButtonAvailable => _locations.SystemButton >= 0;

    /// <summary>Returns whether trigger is pressed enough</summary>
    public bool TriggerButton => _locations.TriggerButton >= 0 && _dataBool[_locations.TriggerButton];
    /// <summary>Returns whether the TriggerButton is available</summary>
    public bool TriggerButtonAvailable => _locations.TriggerButton >= 0;

    /// <summary>Returns whether trigger button is touched</summary>
    /// On Vive Wands which do not have this in hardware this is emulated as
    /// true when the trigger is pressed a little bit.
    public bool TriggerTouch => _locations.TriggerTouch >= 0
        ? _dataBool[_locations.TriggerTouch]
        : TriggerAvailable && Trigger > 0;
    /// <summary>Returns whether TriggerTouch is available in hardware</summary>
    public bool TriggerTouchAvailable => _locations.TriggerTouch >= 0;

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

                _locations = new();
            }
            return;
        }

        if (_localUniqueId != device.LocallyUniqueId)
        {
            _localUniqueId = device.LocallyUniqueId;
            Characteristics = device.Characteristics;
            Name = device.Name;
            DeviceInfoChanged = true;

            // ADDING_NEW_TYPE:
            // add if branch here
            if (_dataQuaternion == null || _dataQuaternion.Length != device.Quaternion.Length) _dataQuaternion = new Quaternion[device.Quaternion.Length];
            if (_dataVector3 == null || _dataVector3.Length != device.Vector3.Length) _dataVector3 = new Vector3[device.Vector3.Length];
            if (_dataVector2 == null || _dataVector2.Length != device.Vector2.Length) _dataVector2 = new Vector2[device.Vector2.Length];
            if (_dataFloat == null || _dataFloat.Length != device.Float.Length) _dataFloat = new float[device.Float.Length];
            if (_dataBool == null || _dataBool.Length != device.Bool.Length) _dataBool = new bool[device.Bool.Length];
            if (_dataUint == null || _dataUint.Length != device.Uint.Length) _dataUint = new uint[device.Uint.Length];

            // ADDING_NEW_USAGE:
            // add if branch to relevant for loop
            for (var i = 0; i < _dataQuaternion.Length; ++i)
            {
                var name = device.Quaternion[i].name;
                if (name == "DeviceRotation") _locations.DeviceRotation = i;
                else if (name == "PointerRotation") _locations.PointerRotation = i;
                else Debug.Log($"Unknown usage device of type Quaternion with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Vector3.Length; ++i)
            {
                var name = device.Vector3[i].name;
                if (name == "DeviceAngularVelocity") _locations.DeviceAngularVelocity = i;
                else if (name == "DevicePosition") _locations.DevicePosition = i;
                else if (name == "DeviceVelocity") _locations.DeviceVelocity = i;
                else if (name == "PointerAngularVelocity") _locations.PointerAngularVelocity = i;
                else if (name == "PointerPosition") _locations.PointerPosition = i;
                else if (name == "PointerVelocity") _locations.PointerVelocity = i;
                else Debug.Log($"Unknown usage device of type Vector3 with name {name} on device {device.Name}");
            }

            for (var i = 0; i < device.Vector2.Length; ++i)
            {
                var name = device.Vector2[i].name;
                if (name == "Primary2DAxis") _locations.Primary2DAxis = i;
                else Debug.Log($"Unknown usage device of type Vector2 with name {name} on device {device.Name}");
            }
            for (var i = 0; i < device.Float.Length; ++i)
            {
                var name = device.Float[i].name;
                if (name == "Grip") _locations.Grip = i;
                else if (name == "Trigger") _locations.Trigger = i;
                else Debug.Log($"Unknown usage device of type float with name {name} on device {device.Name}");
            }
            for (var i = 0; i < device.Bool.Length; ++i)
            {
                var name = device.Bool[i].name;
                if (name == "GripButton") _locations.GripButton = i;
                else if (name == "IsTracked") _locations.IsTracked = i;
                // Note: I merged Oculus Touch's MenuButton and Vive's Menu into MenuButton only
                else if (name == "Menu" || name == "MenuButton") _locations.MenuButton = i;
                else if (name == "Primary2DAxisClick") _locations.Primary2DAxisClick = i;
                else if (name == "Primary2DAxisTouch") _locations.Primary2DAxisTouch = i;
                else if (name == "PrimaryButton") _locations.PrimaryButton = i;
                else if (name == "PrimaryTouch") _locations.PrimaryTouch = i;
                else if (name == "SecondaryButton") _locations.SecondaryButton = i;
                else if (name == "SecondaryTouch") _locations.SecondaryTouch = i;
                else if (name == "SystemButton") _locations.SystemButton = i;
                else if (name == "TriggerButton") _locations.TriggerButton = i;
                else if (name == "TriggerTouch") _locations.TriggerTouch = i;
                else Debug.Log($"Unknown usage device of type bool with name {name} on device {device.Name}");
            }
            for (var i = 0; i < device.Uint.Length; ++i)
            {
                var name = device.Uint[i].name;
                if (name == "TrackingState") _locations.TrackingState = i;
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
        }

        // ADDING_NEW_TYPE:
        // add for loop here
        for (var i = 0; i < _dataQuaternion.Length; ++i) device.Device.TryGetFeatureValue(device.Quaternion[i], out _dataQuaternion[i]);
        for (var i = 0; i < _dataVector3.Length; ++i) device.Device.TryGetFeatureValue(device.Vector3[i], out _dataVector3[i]);
        for (var i = 0; i < _dataVector2.Length; ++i) device.Device.TryGetFeatureValue(device.Vector2[i], out _dataVector2[i]);
        for (var i = 0; i < _dataFloat.Length; ++i) device.Device.TryGetFeatureValue(device.Float[i], out _dataFloat[i]);
        for (var i = 0; i < _dataBool.Length; ++i) device.Device.TryGetFeatureValue(device.Bool[i], out _dataBool[i]);
        for (var i = 0; i < _dataUint.Length; ++i) device.Device.TryGetFeatureValue(device.Uint[i], out _dataUint[i]);
    }
}
