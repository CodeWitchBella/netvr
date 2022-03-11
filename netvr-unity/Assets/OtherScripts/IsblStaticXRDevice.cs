using System;
using System.IO;
using System.Text.Json.Nodes;
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
/// | Usage                                   |Vive |Quest2|Vive   |
/// |                                         |Wand |Touch |HeadSet|
/// |-----------------------------------------|-----|------|-------|
/// | DeviceAngularVelocity(Vector3)          | ✅ |  ✅  |  ✅   |
/// | DevicePosition(Vector3)                 | ✅ |  ✅  |  ✅   |
/// | DeviceRotation(Quaternion)              | ✅ |  ✅  |  ✅   |
/// | DeviceVelocity(Vector3)                 | ✅ |  ✅  |  ✅   |
/// | Grip(float)                             | ✅ |  ✅  |  ❌   |
/// | GripButton(bool)                        | ✅ |  ✅  |  ❌   |
/// | IsTracked(bool)                         | ✅ |  ✅  |  ✅   |
/// | Menu(bool)                              | ❌ |  ✅  |  ❌   |
/// | MenuButton(bool)                        | ✅ |  ❌  |  ❌   |
/// | PointerAngularVelocity(Vector3)         | ✅ |  ✅  |  ❌   |
/// | PointerPosition(Vector3)                | ✅ |  ✅  |  ❌   |
/// | PointerRotation(Quaternion)             | ✅ |  ✅  |  ❌   |
/// | PointerVelocity(Vector3)                | ✅ |  ✅  |  ❌   |
/// | Primary2DAxis(Vector2)                  | ✅ |  ✅  |  ❌   |
/// | Primary2DAxisClick(bool)                | ✅ |  ✅  |  ❌   |
/// | Primary2DAxisTouch(bool)                | ✅ |  ✅  |  ❌   |
/// | PrimaryButton(bool)                     | ❌ |  ✅  |  ❌   |
/// | PrimaryTouch(bool)                      | ❌ |  ✅  |  ❌   |
/// | SecondaryButton(bool)                   | ❌ |  ✅  |  ❌   |
/// | SecondaryTouch(bool)                    | ❌ |  ✅  |  ❌   |
/// | SystemButton(bool)                      | ✅ |  ❌  |  ❌   |
/// | TrackingState(uint)                     | ✅ |  ✅  |  ✅   |
/// | Trigger(float)                          | ✅ |  ✅  |  ❌   |
/// | TriggerButton(bool)                     | ✅ |  ✅  |  ❌   |
/// | TriggerTouch(bool)                      | ❌ |  ✅  |  ❌   |
/// | CenterEyeRotation(Quaternion)           | ❌ |  ❌  |  ✅   |
/// | ColorCameraRotation(Quaternion)         | ❌ |  ❌  |  ❌   |
/// | LeftEyeRotation(Quaternion)             | ❌ |  ❌  |  ✅   |
/// | RightEyeRotation(Quaternion)            | ❌ |  ❌  |  ✅   |
/// | CenterEyeAcceleration(Vector3)          | ❌ |  ❌  |  ❌   |
/// | CenterEyeAngularAcceleration(Vector3)   | ❌ |  ❌  |  ❌   |
/// | CenterEyeAngularVelocity(Vector3)       | ❌ |  ❌  |  ✅   |
/// | CenterEyePosition(Vector3)              | ❌ |  ❌  |  ✅   |
/// | CenterEyeVelocity(Vector3)              | ❌ |  ❌  |  ✅   |
/// | ColorCameraAcceleration(Vector3)        | ❌ |  ❌  |  ❌   |
/// | ColorCameraAngularAcceleration(Vector3) | ❌ |  ❌  |  ❌   |
/// | ColorCameraAngularVelocity(Vector3)     | ❌ |  ❌  |  ❌   |
/// | ColorCameraPosition(Vector3)            | ❌ |  ❌  |  ❌   |
/// | ColorCameraVelocity(Vector3)            | ❌ |  ❌  |  ❌   |
/// | DeviceAcceleration(Vector3)             | ❌ |  ❌  |  ❌   |
/// | DeviceAngularAcceleration(Vector3)      | ❌ |  ❌  |  ❌   |
/// | LeftEyeAcceleration(Vector3)            | ❌ |  ❌  |  ❌   |
/// | LeftEyeAngularAcceleration(Vector3)     | ❌ |  ❌  |  ❌   |
/// | LeftEyeAngularVelocity(Vector3)         | ❌ |  ❌  |  ✅   |
/// | LeftEyePosition(Vector3)                | ❌ |  ❌  |  ✅   |
/// | LeftEyeVelocity(Vector3)                | ❌ |  ❌  |  ✅   |
/// | RightEyeAcceleration(Vector3)           | ❌ |  ❌  |  ❌   |
/// | RightEyeAngularAcceleration(Vector3)    | ❌ |  ❌  |  ❌   |
/// | RightEyeAngularVelocity(Vector3)        | ❌ |  ❌  |  ✅   |
/// | RightEyePosition(Vector3)               | ❌ |  ❌  |  ✅   |
/// | RightEyeVelocity(Vector3)               | ❌ |  ❌  |  ✅   |
/// | Secondary2DAxis(Vector2)                | ❌ |  ❌  |  ❌   |
/// | BatteryLevel(float)                     | ❌ |  ❌  |  ❌   |
/// | Secondary2DAxisClick(bool)              | ❌ |  ❌  |  ❌   |
/// | Secondary2DAxisTouch(bool)              | ❌ |  ❌  |  ❌   |
/// | UserPresence(bool)                      | ❌ |  ❌  |  ✅   |
/// | HandData(Hand)                          | ❌ |  ❌  |  ❌   |
/// | EyesData(Eyes)                          | ❌ |  ❌  |  ❌   |
// ADDING_NEW_USAGE: update this table
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
    // TODO: Netcode is not implemented yet for this. Search for HAND_NETCODE
    // when implementing.
    Hand[] _dataHand;
    // TODO: Since I do not have eye tracking device I could not test this type and
    // therefore did not code netcode for this. Works only locally. Search for
    // EYES_NETCODE when implementing.
    Eyes[] _dataEyes;

    public string Name { get; private set; }
    public UInt16 LocallyUniqueId;
    public bool IsLocal;
    public InputDeviceCharacteristics Characteristics { get; private set; }
    public StaticHaptics Haptics;
    Locations _locations = new();

    /// <summary>
    /// Denotes whether information stored outside of Data chaned
    /// </summary>
    ///
    /// True means that the data needs to be transmitted to the server.
    public bool DeviceInfoChanged;
    #endregion

    // HAND_NETCODE: increment by one
    // EYES_NETCODE: increment by one
    // ADDING_NEW_TYPE:
    // Increment this by one
    const int TypeCount = 6;

    #region (De)Serialization
    public int CalculateSerializationSize(bool contentOnly = false)
    {
        if (_dataQuaternion == null) return TypeCount;
        var contentBytes =
            Isbl.NetUtils.Count7BitEncodedIntBytes(LocallyUniqueId)
            + Isbl.NetUtils.CountArrayEncodingBytes(_dataQuaternion.Length, 4 * 3)
            + Isbl.NetUtils.CountArrayEncodingBytes(_dataVector3.Length, 4 * 3)
            + Isbl.NetUtils.CountArrayEncodingBytes(_dataVector2.Length, 4 * 2)
            + Isbl.NetUtils.CountArrayEncodingBytes(_dataFloat.Length, 4)
            + Isbl.NetUtils.CountArrayEncodingBytes(_dataBool.Length, 1)
            + Isbl.NetUtils.CountArrayEncodingBytes(_dataUint.Length, 4)
            // HAND_NETCODE: add size calculation
            // EYES_NETCODE: add size calculation
            // ADDING_NEW_TYPE:
            // add line for calculating serialization size above this comment
            ;
        if (contentOnly) return contentBytes;
        return Isbl.NetUtils.Count7BitEncodedIntBytes(contentBytes) + contentBytes;
    }

    public int DeSerializeData(byte[] source, int offset)
    {
        MemoryStream stream = new(source);
        stream.Position = offset;
        BinaryReader reader = new(stream);

        {
            var len = Isbl.NetUtils.Read7BitEncodedInt(reader);
            if (_dataQuaternion?.Length == len)
                for (int i = 0; i < len; ++i) _dataQuaternion[i] = Quaternion.Euler((float)(reader.ReadSingle() / Math.PI * 180f), (float)(reader.ReadSingle() / Math.PI * 180f), (float)(reader.ReadSingle() / Math.PI * 180f));
            else reader.BaseStream.Seek(len * 4 * 3, SeekOrigin.Current);
        }

        {
            var len = Isbl.NetUtils.Read7BitEncodedInt(reader);
            if (_dataVector3?.Length == len)
                for (int i = 0; i < len; ++i) _dataVector3[i] = new Vector3(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            else reader.BaseStream.Seek(len * 4 * 3, SeekOrigin.Current);
        }

        {
            var len = Isbl.NetUtils.Read7BitEncodedInt(reader);
            if (_dataVector2?.Length == len)
                for (int i = 0; i < len; ++i) _dataVector2[i] = new Vector2(reader.ReadSingle(), reader.ReadSingle());
            else reader.BaseStream.Seek(len * 4 * 2, SeekOrigin.Current);
        }

        {
            var len = Isbl.NetUtils.Read7BitEncodedInt(reader);
            if (_dataFloat?.Length == len)
                for (int i = 0; i < len; ++i) _dataFloat[i] = reader.ReadSingle();
            else reader.BaseStream.Seek(len * 4, SeekOrigin.Current);
        }

        {
            var len = Isbl.NetUtils.Read7BitEncodedInt(reader);
            if (_dataBool?.Length == len)
                for (int i = 0; i < len; ++i) _dataBool[i] = reader.ReadBoolean();
            else reader.BaseStream.Seek(len * 1, SeekOrigin.Current);
        }

        {
            var len = Isbl.NetUtils.Read7BitEncodedInt(reader);
            if (_dataUint?.Length == len)
                for (int i = 0; i < len; ++i) _dataUint[i] = reader.ReadUInt32();
            else reader.BaseStream.Seek(len * 4, SeekOrigin.Current);
        }

        // HAND_NETCODE: add deserialization block
        // EYES_NETCODE: add deserialization block

        // ADDING_NEW_TYPE:
        // Add deserialization code above this comment

        return (int)stream.Position - offset;
    }

    public int SerializeData(byte[] target, int offset)
    {
        MemoryStream stream = new(target);
        stream.Position = offset;
        BinaryWriter writer = new(stream);
        Isbl.NetUtils.Write7BitEncodedInt(writer, CalculateSerializationSize(contentOnly: true));
        Isbl.NetUtils.Write7BitEncodedInt(writer, LocallyUniqueId);
        if (_dataQuaternion == null)
        {
            throw new Exception("You should not serialize empty devices");
        }

        Isbl.NetUtils.Write7BitEncodedInt(writer, _dataQuaternion.Length);
        foreach (var el in _dataQuaternion)
        {
            writer.Write((float)(el.eulerAngles.x / 180.0 * Math.PI));
            writer.Write((float)(el.eulerAngles.y / 180.0 * Math.PI));
            writer.Write((float)(el.eulerAngles.z / 180.0 * Math.PI));
        }

        Isbl.NetUtils.Write7BitEncodedInt(writer, _dataVector3.Length);
        foreach (var el in _dataVector3)
        {
            writer.Write(el.x);
            writer.Write(el.y);
            writer.Write(el.z);
        }

        Isbl.NetUtils.Write7BitEncodedInt(writer, _dataVector2.Length);
        foreach (var el in _dataVector2)
        {
            writer.Write(el.x);
            writer.Write(el.y);
        }

        Isbl.NetUtils.Write7BitEncodedInt(writer, _dataFloat.Length);
        foreach (var el in _dataFloat) writer.Write(el);

        Isbl.NetUtils.Write7BitEncodedInt(writer, _dataBool.Length);
        foreach (var el in _dataBool) writer.Write(el);

        Isbl.NetUtils.Write7BitEncodedInt(writer, _dataUint.Length);
        foreach (var el in _dataUint) writer.Write(el);

        // HAND_NETCODE: add serialization block
        // EYES_NETCODE: add serialization block

        // ADDING_NEW_TYPE:
        // Add serialization code above this comment

        return (int)stream.Position - offset;
    }

    public string CSVHeader { get; private set; }

    public string SerializeDataAsCsv()
    {
        System.Text.StringBuilder builder = new();
        StringWriter writer = new(builder, System.Globalization.CultureInfo.InvariantCulture);

        foreach (var el in _dataQuaternion)
        {
            writer.Write((float)(el.eulerAngles.x / 180.0 * Math.PI)); writer.Write(";");
            writer.Write((float)(el.eulerAngles.y / 180.0 * Math.PI)); writer.Write(";");
            writer.Write((float)(el.eulerAngles.z / 180.0 * Math.PI)); writer.Write(";");
        }

        foreach (var el in _dataVector3)
        {
            writer.Write(el.x); writer.Write(";");
            writer.Write(el.y); writer.Write(";");
            writer.Write(el.z); writer.Write(";");
        }

        foreach (var el in _dataVector2)
        {
            writer.Write(el.x); writer.Write(";");
            writer.Write(el.y); writer.Write(";");
        }

        foreach (var el in _dataFloat) { writer.Write(el); writer.Write(";"); }

        foreach (var el in _dataBool) { writer.Write(el); writer.Write(";"); }

        foreach (var el in _dataUint) { writer.Write(el); writer.Write(";"); }

        // HAND_NETCODE: add serialization block
        // EYES_NETCODE: add serialization block

        // ADDING_NEW_TYPE:
        // Add serialization code above this comment

        return builder.ToString();
    }

    public JsonObject SerializeConfiguration()
    {
        JsonArray characteristics = new();
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

        return Isbl.NetUtils.JsonFromObject(new
        {
            locations = Isbl.NetUtils.ToJsonCamelCase(_locations),
            name = Name,
            characteristics,
            localId = LocallyUniqueId,
            haptics = Haptics != null ? Isbl.NetUtils.ToJsonCamelCase(Haptics) : null,
            lengths = new
            {
                quaternion = _dataQuaternion?.Length ?? 0,
                vector3 = _dataVector3?.Length ?? 0,
                vector2 = _dataVector2?.Length ?? 0,
                @float = _dataFloat?.Length ?? 0,
                @bool = _dataBool?.Length ?? 0,
                @uint = _dataUint?.Length ?? 0,
                hand = _dataHand?.Length ?? 0, // HAND_NETCODE: already done
                eyes = _dataEyes?.Length ?? 0, // EYES_NETCODE: already done
                // ADDING_NEW_TYPE:
                // Add line here
            }
        });
    }

    public void DeSerializeConfiguration(Newtonsoft.Json.Linq.JObject message)
    {
        Characteristics = 0;
        foreach (var c in message.Value<Newtonsoft.Json.Linq.JArray>("characteristics"))
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

        var haptics = message.Value<Newtonsoft.Json.Linq.JObject>("haptics");
        Haptics = haptics != null ? new StaticHaptics(haptics) : null;

        _locations = Isbl.NetUtils.FromJObjectCamelCase<Locations>(message.Value<Newtonsoft.Json.Linq.JObject>("locations"));
        Name = message.Value<string>("name");
        LocallyUniqueId = message.Value<UInt16>("localId");
        var lengths = message.Value<Newtonsoft.Json.Linq.JObject>("lengths");
        _dataQuaternion = new Quaternion[lengths.Value<int>("quaternion")];
        _dataVector3 = new Vector3[lengths.Value<int>("vector3")];
        _dataVector2 = new Vector2[lengths.Value<int>("vector2")];
        _dataFloat = new float[lengths.Value<int>("float")];
        _dataBool = new bool[lengths.Value<int>("bool")];
        _dataUint = new uint[lengths.Value<int>("uint")];
        _dataHand = new Hand[lengths.Value<int>("hand")]; // HAND_NETCODE: already implemented
        _dataEyes = new Eyes[lengths.Value<int>("eyes")]; // EYES_NETCODE: already implemented
        // ADDING_NEW_TYPE:
        // Add line here
    }
    #endregion

    public bool HasData => _dataQuaternion != null;
    /// <summary>
    /// Copy-paste of HapticCapabilities because that does not allow setting
    /// values. Not even in constructor. All fields are the same as those of
    /// HapticCapabilities.
    /// </summary>
    /// <seealso cref="UnityEngine.XR.HapticCapabilities"/>
    public class StaticHaptics
    {
        public uint NumChannels;
        public bool SupportsImpulse;
        public bool SupportsBuffer;
        public uint BufferFrequencyHz;
        public uint BufferMaxSize;
        public uint BufferOptimalSize;
        public StaticHaptics(HapticCapabilities caps)
        {
            NumChannels = caps.numChannels;
            SupportsImpulse = caps.supportsImpulse;
            SupportsBuffer = caps.supportsBuffer;
            BufferFrequencyHz = caps.bufferFrequencyHz;
            BufferMaxSize = caps.bufferMaxSize;
            BufferOptimalSize = caps.bufferOptimalSize;
        }
        public StaticHaptics(Newtonsoft.Json.Linq.JObject caps)
        {
            NumChannels = caps.Value<uint>("numChannels");
            SupportsImpulse = caps.Value<bool>("supportsImpulse");
            SupportsBuffer = caps.Value<bool>("supportsBuffer");
            BufferFrequencyHz = caps.Value<uint>("bufferFrequencyHz");
            BufferMaxSize = caps.Value<uint>("bufferMaxSize");
            BufferOptimalSize = caps.Value<uint>("bufferOptimalSize");
        }
    };

    class Locations
    {
        // ADDING_NEW_USAGE:
        // (search for ADDING_NEW_USAGE to see all relevant markers)
        // add new member here
        public int DeviceRotation = -1;//Quaternion
        public int PointerRotation = -1;//Quaternion
        public int CenterEyeRotation = -1;//Quaternion
        public int ColorCameraRotation = -1;//Quaternion
        public int LeftEyeRotation = -1;//Quaternion
        public int RightEyeRotation = -1;//Quaternion
        public int DeviceAngularVelocity = -1;//Vector3
        public int DevicePosition = -1;//Vector3
        public int DeviceVelocity = -1;//Vector3
        public int PointerAngularVelocity = -1;//Vector3
        public int PointerPosition = -1;//Vector3
        public int PointerVelocity = -1;//Vector3
        public int CenterEyeAcceleration = -1;//Vector3
        public int CenterEyeAngularAcceleration = -1;//Vector3
        public int CenterEyeAngularVelocity = -1;//Vector3
        public int CenterEyePosition = -1;//Vector3
        public int CenterEyeVelocity = -1;//Vector3
        public int ColorCameraAcceleration = -1;//Vector3
        public int ColorCameraAngularAcceleration = -1;//Vector3
        public int ColorCameraAngularVelocity = -1;//Vector3
        public int ColorCameraPosition = -1;//Vector3
        public int ColorCameraVelocity = -1;//Vector3
        public int DeviceAcceleration = -1;//Vector3
        public int DeviceAngularAcceleration = -1;//Vector3
        public int LeftEyeAcceleration = -1;//Vector3
        public int LeftEyeAngularAcceleration = -1;//Vector3
        public int LeftEyeAngularVelocity = -1;//Vector3
        public int LeftEyePosition = -1;//Vector3
        public int LeftEyeVelocity = -1;//Vector3
        public int RightEyeAcceleration = -1;//Vector3
        public int RightEyeAngularAcceleration = -1;//Vector3
        public int RightEyeAngularVelocity = -1;//Vector3
        public int RightEyePosition = -1;//Vector3
        public int RightEyeVelocity = -1;//Vector3
        public int Primary2DAxis = -1;//Vector2
        public int Secondary2DAxis = -1;//Vector2
        public int Grip = -1;//float
        public int Trigger = -1;//float
        public int BatteryLevel = -1;//float
        public int TrackingState = -1;//uint
        public int GripButton = -1;//bool
        public int IsTracked = -1;//bool
        public int MenuButton = -1;//bool
        public int Primary2DAxisClick = -1;//bool
        public int Primary2DAxisTouch = -1;//bool
        public int PrimaryButton = -1;//bool
        public int PrimaryTouch = -1;//bool
        public int SecondaryButton = -1;//bool
        public int SecondaryTouch = -1;//bool
        public int SystemButton = -1;//bool
        public int TriggerButton = -1;//bool
        public int TriggerTouch = -1;//bool
        public int Secondary2DAxisClick = -1;//bool
        public int Secondary2DAxisTouch = -1;//bool
        public int UserPresence = -1;//bool
        public int HandData = -1;//Hand
        public int EyesData = -1;//Eyes
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

    /// <summary>Rotation of center eye of this device</summary>
    public Quaternion CenterEyeRotation => _locations.CenterEyeRotation >= 0 ? _dataQuaternion[_locations.CenterEyeRotation] : Quaternion.identity;
    /// <summary>Returns whether the CenterEyeRotationAvailable is available</summary>
    public bool CenterEyeRotationAvailable => _locations.CenterEyeRotation >= 0;

    /// <summary>Rotation of color camera on this device</summary>
    public Quaternion ColorCameraRotation => _locations.ColorCameraRotation >= 0 ? _dataQuaternion[_locations.ColorCameraRotation] : Quaternion.identity;
    /// <summary>Returns whether the ColorCameraRotationAvailable is available</summary>
    public bool ColorCameraRotationAvailable => _locations.ColorCameraRotation >= 0;

    /// <summary>Rotation of left eye of this device</summary>
    public Quaternion LeftEyeRotation => _locations.LeftEyeRotation >= 0 ? _dataQuaternion[_locations.LeftEyeRotation] : Quaternion.identity;
    /// <summary>Returns whether the LeftEyeRotationAvailable is available</summary>
    public bool LeftEyeRotationAvailable => _locations.LeftEyeRotation >= 0;

    /// <summary>Rotation of right eye of this device</summary>
    public Quaternion RightEyeRotation => _locations.RightEyeRotation >= 0 ? _dataQuaternion[_locations.RightEyeRotation] : Quaternion.identity;
    /// <summary>Returns whether the RightEyeRotationAvailable is available</summary>
    public bool RightEyeRotationAvailable => _locations.RightEyeRotation >= 0;

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

    /// <summary>Rate of change of CenterEyeVelocity</summary>
    public Vector3 CenterEyeAcceleration => _locations.CenterEyeAcceleration >= 0 ? _dataVector3[_locations.CenterEyeAcceleration] : Vector3.zero;
    /// <summary>Returns whether the CenterEyeAcceleration is available</summary>
    public bool CenterEyeAccelerationAvailable => _locations.CenterEyeAcceleration >= 0;

    /// <summary>Rate of change of CenterEyeAngularVelocity</summary>
    public Vector3 CenterEyeAngularAcceleration => _locations.CenterEyeAngularAcceleration >= 0 ? _dataVector3[_locations.CenterEyeAngularAcceleration] : Vector3.zero;
    /// <summary>Returns whether the CenterEyeAngularAcceleration is available</summary>
    public bool CenterEyeAngularAccelerationAvailable => _locations.CenterEyeAngularAcceleration >= 0;

    /// <summary>Rate of change of CenterEyeRotation</summary>
    public Vector3 CenterEyeAngularVelocity => _locations.CenterEyeAngularVelocity >= 0 ? _dataVector3[_locations.CenterEyeAngularVelocity] : Vector3.zero;
    /// <summary>Returns whether the CenterEyeAngularVelocity is available</summary>
    public bool CenterEyeAngularVelocityAvailable => _locations.CenterEyeAngularVelocity >= 0;

    /// <summary>Position of center eye of this device</summary>
    public Vector3 CenterEyePosition => _locations.CenterEyePosition >= 0 ? _dataVector3[_locations.CenterEyePosition] : Vector3.zero;
    /// <summary>Returns whether the CenterEyePosition is available</summary>
    public bool CenterEyePositionAvailable => _locations.CenterEyePosition >= 0;

    /// <summary>Rate of change of CenterEyePosition</summary>
    public Vector3 CenterEyeVelocity => _locations.CenterEyeVelocity >= 0 ? _dataVector3[_locations.CenterEyeVelocity] : Vector3.zero;
    /// <summary>Returns whether the CenterEyeVelocity is available</summary>
    public bool CenterEyeVelocityAvailable => _locations.CenterEyeVelocity >= 0;

    /// <summary>Rate of change of ColorCameraPosition</summary>
    public Vector3 ColorCameraAcceleration => _locations.ColorCameraAcceleration >= 0 ? _dataVector3[_locations.ColorCameraAcceleration] : Vector3.zero;
    /// <summary>Returns whether the ColorCameraAcceleration is available</summary>
    public bool ColorCameraAccelerationAvailable => _locations.ColorCameraAcceleration >= 0;

    /// <summary>Rate of change of ColorCameraAngularVelocity</summary>
    public Vector3 ColorCameraAngularAcceleration => _locations.ColorCameraAngularAcceleration >= 0 ? _dataVector3[_locations.ColorCameraAngularAcceleration] : Vector3.zero;
    /// <summary>Returns whether the ColorCameraAngularAcceleration is available</summary>
    public bool ColorCameraAngularAccelerationAvailable => _locations.ColorCameraAngularAcceleration >= 0;

    /// <summary>Rate of change of ColorCameraRotation</summary>
    public Vector3 ColorCameraAngularVelocity => _locations.ColorCameraAngularVelocity >= 0 ? _dataVector3[_locations.ColorCameraAngularVelocity] : Vector3.zero;
    /// <summary>Returns whether the ColorCameraAngularVelocity is available</summary>
    public bool ColorCameraAngularVelocityAvailable => _locations.ColorCameraAngularVelocity >= 0;

    /// <summary>Position of color camera attached to this device</summary>
    public Vector3 ColorCameraPosition => _locations.ColorCameraPosition >= 0 ? _dataVector3[_locations.ColorCameraPosition] : Vector3.zero;
    /// <summary>Returns whether the ColorCameraPosition is available</summary>
    public bool ColorCameraPositionAvailable => _locations.ColorCameraPosition >= 0;

    /// <summary>Rate of change of ColorCameraPosition</summary>
    public Vector3 ColorCameraVelocity => _locations.ColorCameraVelocity >= 0 ? _dataVector3[_locations.ColorCameraVelocity] : Vector3.zero;
    /// <summary>Returns whether the ColorCameraVelocity is available</summary>
    public bool ColorCameraVelocityAvailable => _locations.ColorCameraVelocity >= 0;

    /// <summary>Rate of change of DeviceVelocity</summary>
    public Vector3 DeviceAcceleration => _locations.DeviceAcceleration >= 0 ? _dataVector3[_locations.DeviceAcceleration] : Vector3.zero;
    /// <summary>Returns whether the DeviceAcceleration is available</summary>
    public bool DeviceAccelerationAvailable => _locations.DeviceAcceleration >= 0;

    /// <summary>Rate of change of DeviceAngularVelocity</summary>
    public Vector3 DeviceAngularAcceleration => _locations.DeviceAngularAcceleration >= 0 ? _dataVector3[_locations.DeviceAngularAcceleration] : Vector3.zero;
    /// <summary>Returns whether the DeviceAngularAcceleration is available</summary>
    public bool DeviceAngularAccelerationAvailable => _locations.DeviceAngularAcceleration >= 0;

    /// <summary>Rate of change of LeftEyeVelocity</summary>
    public Vector3 LeftEyeAcceleration => _locations.LeftEyeAcceleration >= 0 ? _dataVector3[_locations.LeftEyeAcceleration] : Vector3.zero;
    /// <summary>Returns whether the LeftEyeAcceleration is available</summary>
    public bool LeftEyeAccelerationAvailable => _locations.LeftEyeAcceleration >= 0;

    /// <summary>Rate of change of LeftEyeAngularVelocity</summary>
    public Vector3 LeftEyeAngularAcceleration => _locations.LeftEyeAngularAcceleration >= 0 ? _dataVector3[_locations.LeftEyeAngularAcceleration] : Vector3.zero;
    /// <summary>Returns whether the LeftEyeAngularAcceleration is available</summary>
    public bool LeftEyeAngularAccelerationAvailable => _locations.LeftEyeAngularAcceleration >= 0;

    /// <summary>Rate of change of LeftEyeRotation</summary>
    public Vector3 LeftEyeAngularVelocity => _locations.LeftEyeAngularVelocity >= 0 ? _dataVector3[_locations.LeftEyeAngularVelocity] : Vector3.zero;
    /// <summary>Returns whether the LeftEyeAngularVelocity is available</summary>
    public bool LeftEyeAngularVelocityAvailable => _locations.LeftEyeAngularVelocity >= 0;

    /// <summary>Position of left eye of this device</summary>
    public Vector3 LeftEyePosition => _locations.LeftEyePosition >= 0 ? _dataVector3[_locations.LeftEyePosition] : Vector3.zero;
    /// <summary>Returns whether the LeftEyePosition is available</summary>
    public bool LeftEyePositionAvailable => _locations.LeftEyePosition >= 0;

    /// <summary>Rate of change of LeftEyePosition</summary>
    public Vector3 LeftEyeVelocity => _locations.LeftEyeVelocity >= 0 ? _dataVector3[_locations.LeftEyeVelocity] : Vector3.zero;
    /// <summary>Returns whether the LeftEyeVelocity is available</summary>
    public bool LeftEyeVelocityAvailable => _locations.LeftEyeVelocity >= 0;

    /// <summary>Rate of change of RightEyeVelocity</summary>
    public Vector3 RightEyeAcceleration => _locations.RightEyeAcceleration >= 0 ? _dataVector3[_locations.RightEyeAcceleration] : Vector3.zero;
    /// <summary>Returns whether the RightEyeAcceleration is available</summary>
    public bool RightEyeAccelerationAvailable => _locations.RightEyeAcceleration >= 0;

    /// <summary>Rate of change of RightEyeAngularVelocity</summary>
    public Vector3 RightEyeAngularAcceleration => _locations.RightEyeAngularAcceleration >= 0 ? _dataVector3[_locations.RightEyeAngularAcceleration] : Vector3.zero;
    /// <summary>Returns whether the RightEyeAngularAcceleration is available</summary>
    public bool RightEyeAngularAccelerationAvailable => _locations.RightEyeAngularAcceleration >= 0;

    /// <summary>Rate of change of RightEyeRotation</summary>
    public Vector3 RightEyeAngularVelocity => _locations.RightEyeAngularVelocity >= 0 ? _dataVector3[_locations.RightEyeAngularVelocity] : Vector3.zero;
    /// <summary>Returns whether the RightEyeAngularVelocity is available</summary>
    public bool RightEyeAngularVelocityAvailable => _locations.RightEyeAngularVelocity >= 0;

    /// <summary>Position of right eye</summary>
    public Vector3 RightEyePosition => _locations.RightEyePosition >= 0 ? _dataVector3[_locations.RightEyePosition] : Vector3.zero;
    /// <summary>Returns whether the RightEyePosition is available</summary>
    public bool RightEyePositionAvailable => _locations.RightEyePosition >= 0;

    /// <summary>Rate of change of RightEyePosition</summary>
    public Vector3 RightEyeVelocity => _locations.RightEyeVelocity >= 0 ? _dataVector3[_locations.RightEyeVelocity] : Vector3.zero;
    /// <summary>Returns whether the RightEyeVelocity is available</summary>
    public bool RightEyeVelocityAvailable => _locations.RightEyeVelocity >= 0;

    /// <summary>Represents direction of primary joystick or where on the
    /// touchpad in case of Vive Wands is user pressing.</summary>
    /// Range is `<-1;1>x<-1;1>`
    public Vector2 Primary2DAxis => _locations.Primary2DAxis >= 0
        ? _dataVector2[_locations.Primary2DAxis]
        : Vector2.zero;
    /// <summary>Returns whether the Primary2DAxis is available</summary>
    public bool Primary2DAxisAvailable => _locations.Primary2DAxis >= 0;

    /// <summary>Represents direction of secondary joystick or where on the
    /// touchpad in case of Vive Wands is user pressing.</summary>
    /// Range is `<-1;1>x<-1;1>`
    public Vector2 Secondary2DAxis => _locations.Secondary2DAxis >= 0
        ? _dataVector2[_locations.Secondary2DAxis]
        : Vector2.zero;
    /// <summary>Returns whether the Secondary2DAxis is available</summary>
    public bool Secondary2DAxisAvailable => _locations.Secondary2DAxis >= 0;

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

    /// <summary>Returns battery charge status</summary>
    /// Fallback value is 0.5 (the least special charge status)
    public float BatteryLevel => _locations.BatteryLevel >= 0 ? _dataFloat[_locations.BatteryLevel] : 0.5f;
    /// <summary>Returns whether the Trigger is available</summary>
    public bool BatteryLevelAvailable => _locations.BatteryLevel >= 0;

    /// <summary>Returns raw tracking state with more detail than IsTracked</summary>
    /// I did not research details of this field and therefore am unsure of how
    /// exactly it represents its values.
    public InputTrackingState TrackingState => _locations.TrackingState >= 0 ? (InputTrackingState)_dataUint[_locations.TrackingState] : 0;
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

    /// <summary>Whether or not is secondary joystick/touchpad pressed in</summary>
    public bool Secondary2DAxisClick => _locations.Secondary2DAxisClick >= 0 && _dataBool[_locations.Secondary2DAxisClick];
    /// <summary>Returns whether Secondary2DAxisClick is available in hardware</summary>
    public bool Secondary2DAxisClickAvailable => _locations.Secondary2DAxisClick >= 0;

    /// <summary>Whether or not is user's finger touching secondary joystick/touchpad</summary>
    public bool Secondary2DAxisTouch => _locations.Secondary2DAxisTouch >= 0 ? _dataBool[_locations.Secondary2DAxisTouch] : Secondary2DAxisClick;
    /// <summary>Returns whether Secondary2DAxisTouch is available in hardware</summary>
    public bool Secondary2DAxisTouchAvailable => _locations.Secondary2DAxisTouch >= 0;

    /// <summary>Whether user is wearing the headset</summary>
    ///
    /// Fallback (value if not available) for this is true.
    public bool UserPresence => _locations.UserPresence < 0 || _dataBool[_locations.UserPresence];
    /// <summary>Returns whether UserPresence is available in hardware</summary>
    public bool UserPresenceAvailable => _locations.UserPresence >= 0;

    // HAND_NETCODE: update summary
    /// <summary>Local-only data about hand tracking</summary>
    public Hand HandData => _locations.HandData >= 0 ? _dataHand[_locations.HandData] : new Hand();
    /// <summary>Returns whether HandData is available in hardware</summary>
    public bool HandDataAvailable => _locations.HandData >= 0;

    // EYES_NETCODE: update summary
    /// <summary>Local-only eye tracking data</summary>
    public Eyes EyesData => _locations.EyesData >= 0 ? _dataEyes[_locations.EyesData] : new Eyes();
    /// <summary>Returns whether EyesData is available in hardware</summary>
    public bool EyesDataAvailable => _locations.EyesData >= 0;

    /// <summary>
    /// Reads data from device to update internal state
    /// </summary>
    public void UpdateFromDevice(IsblXRDevice device)
    {
        IsLocal = true;
        if (device == null)
        {
            if (LocallyUniqueId != 0)
            {
                Name = "";
                Characteristics = 0;
                LocallyUniqueId = 0;
                Haptics = null;
                DeviceInfoChanged = true;

                // ADDING_NEW_TYPE:
                // add if branch here
                _dataQuaternion = null;
                _dataVector3 = null;
                _dataVector2 = null;
                _dataFloat = null;
                _dataBool = null;
                _dataUint = null;
                _dataHand = null; // HAND_NETCODE: already implemented
                _dataEyes = null; // EYES_NETCODE: already implemented

                _locations = new();
            }
            return;
        }

        if (LocallyUniqueId != device.LocallyUniqueId)
        {
            LocallyUniqueId = device.LocallyUniqueId;
            Characteristics = device.Characteristics;
            Name = device.Name;
            Haptics = device.Haptics.HasValue ? new(device.Haptics.Value) : null;
            DeviceInfoChanged = true;

            // ADDING_NEW_TYPE:
            // add if branch here
            if (_dataQuaternion == null || _dataQuaternion.Length != device.Quaternion.Length) _dataQuaternion = new Quaternion[device.Quaternion.Length];
            if (_dataVector3 == null || _dataVector3.Length != device.Vector3.Length) _dataVector3 = new Vector3[device.Vector3.Length];
            if (_dataVector2 == null || _dataVector2.Length != device.Vector2.Length) _dataVector2 = new Vector2[device.Vector2.Length];
            if (_dataFloat == null || _dataFloat.Length != device.Float.Length) _dataFloat = new float[device.Float.Length];
            if (_dataBool == null || _dataBool.Length != device.Bool.Length) _dataBool = new bool[device.Bool.Length];
            if (_dataUint == null || _dataUint.Length != device.Uint.Length) _dataUint = new uint[device.Uint.Length];
            if (_dataHand == null || _dataHand.Length != device.Hand.Length) _dataHand = new Hand[device.Hand.Length]; // HAND_NETCODE: already implemented
            if (_dataEyes == null || _dataEyes.Length != device.Eyes.Length) _dataEyes = new Eyes[device.Eyes.Length]; // EYES_NETCODE: already implemented

            // ADDING_NEW_USAGE:
            // add if branch to relevant for loop
            for (var i = 0; i < _dataQuaternion.Length; ++i)
            {
                var name = device.Quaternion[i].name;
                if (name == "DeviceRotation") _locations.DeviceRotation = i;
                else if (name == "PointerRotation") _locations.PointerRotation = i;
                else if (name == "CenterEyeRotation") _locations.CenterEyeRotation = i;
                else if (name == "ColorCameraRotation") _locations.ColorCameraRotation = i;
                else if (name == "LeftEyeRotation") _locations.LeftEyeRotation = i;
                else if (name == "RightEyeRotation") _locations.RightEyeRotation = i;
                else Debug.Log($"Unknown usage device of type Quaternion with name {name} on device {device.Name}");
                CSVHeader += $"q.{name}.x;q.{name}.y;q.{name}.z;";
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
                else if (name == "CenterEyeAcceleration") _locations.CenterEyeAcceleration = i;
                else if (name == "CenterEyeAngularAcceleration") _locations.CenterEyeAngularAcceleration = i;
                else if (name == "CenterEyeAngularVelocity") _locations.CenterEyeAngularVelocity = i;
                else if (name == "CenterEyePosition") _locations.CenterEyePosition = i;
                else if (name == "CenterEyeVelocity") _locations.CenterEyeVelocity = i;
                else if (name == "ColorCameraAcceleration") _locations.ColorCameraAcceleration = i;
                else if (name == "ColorCameraAngularAcceleration") _locations.ColorCameraAngularAcceleration = i;
                else if (name == "ColorCameraAngularVelocity") _locations.ColorCameraAngularVelocity = i;
                else if (name == "ColorCameraPosition") _locations.ColorCameraPosition = i;
                else if (name == "ColorCameraVelocity") _locations.ColorCameraVelocity = i;
                else if (name == "DeviceAcceleration") _locations.DeviceAcceleration = i;
                else if (name == "DeviceAngularAcceleration") _locations.DeviceAngularAcceleration = i;
                else if (name == "LeftEyeAcceleration") _locations.LeftEyeAcceleration = i;
                else if (name == "LeftEyeAngularAcceleration") _locations.LeftEyeAngularAcceleration = i;
                else if (name == "LeftEyeAngularVelocity") _locations.LeftEyeAngularVelocity = i;
                else if (name == "LeftEyePosition") _locations.LeftEyePosition = i;
                else if (name == "LeftEyeVelocity") _locations.LeftEyeVelocity = i;
                else if (name == "RightEyeAcceleration") _locations.RightEyeAcceleration = i;
                else if (name == "RightEyeAngularAcceleration") _locations.RightEyeAngularAcceleration = i;
                else if (name == "RightEyeAngularVelocity") _locations.RightEyeAngularVelocity = i;
                else if (name == "RightEyePosition") _locations.RightEyePosition = i;
                else if (name == "RightEyeVelocity") _locations.RightEyeVelocity = i;
                else Debug.Log($"Unknown usage device of type Vector3 with name {name} on device {device.Name}");
                CSVHeader += $"v3.{name}.x;v3.{name}.y;v3.{name}.z;";
            }

            for (var i = 0; i < device.Vector2.Length; ++i)
            {
                var name = device.Vector2[i].name;
                if (name == "Primary2DAxis") _locations.Primary2DAxis = i;
                else if (name == "Secondary2DAxis") _locations.Secondary2DAxis = i;
                else Debug.Log($"Unknown usage device of type Vector2 with name {name} on device {device.Name}");
                CSVHeader += $"v2.{name}.x;v2.{name}.y;";
            }
            for (var i = 0; i < device.Float.Length; ++i)
            {
                var name = device.Float[i].name;
                if (name == "Grip") _locations.Grip = i;
                else if (name == "Trigger") _locations.Trigger = i;
                else if (name == "BatteryLevel") _locations.BatteryLevel = i;
                else Debug.Log($"Unknown usage device of type float with name {name} on device {device.Name}");
                CSVHeader += $"f.{name};";
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
                else if (name == "Secondary2DAxisClick") _locations.Secondary2DAxisClick = i;
                else if (name == "Secondary2DAxisTouch") _locations.Secondary2DAxisTouch = i;
                else if (name == "UserPresence") _locations.UserPresence = i;
                else Debug.Log($"Unknown usage device of type bool with name {name} on device {device.Name}");
                CSVHeader += $"b.{name};";
            }
            for (var i = 0; i < device.Uint.Length; ++i)
            {
                var name = device.Uint[i].name;
                if (name == "TrackingState") _locations.TrackingState = i;
                else Debug.Log($"Unknown usage device of type uint with name {name} on device {device.Name}");
                CSVHeader += $"u.{name};";
            }

            for (var i = 0; i < device.Bone.Length; ++i)
            {
                var name = device.Bone[i].name;
                Debug.Log($"Unknown usage device of type Bone with name {name} on device {device.Name}");
                CSVHeader += $"bone.{name};";
            }

            for (var i = 0; i < device.Hand.Length; ++i)
            {
                var name = device.Hand[i].name;
                if (name == "HandData") _locations.HandData = i;
                Debug.Log($"Unknown usage device of type Hand with name {name} on device {device.Name}");
                CSVHeader += $"hand.{name};";
            }

            for (var i = 0; i < device.ByteArray.Length; ++i)
            {
                var name = device.ByteArray[i].name;
                Debug.Log($"Unknown usage device of type Byte with name {name} on device {device.Name}");
                CSVHeader += $"byteArray.{name};";
            }

            for (var i = 0; i < device.Eyes.Length; ++i)
            {
                var name = device.Eyes[i].name;
                if (name == "EyesData") _locations.EyesData = i;
                Debug.Log($"Unknown usage device of type Eyes with name {name} on device {device.Name}");
                CSVHeader += $"eyes.{name};";
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
        for (var i = 0; i < _dataHand.Length; ++i) device.Device.TryGetFeatureValue(device.Hand[i], out _dataHand[i]); // HAND_NETCODE: already implemented
        for (var i = 0; i < _dataEyes.Length; ++i) device.Device.TryGetFeatureValue(device.Eyes[i], out _dataEyes[i]); // EYES_NETCODE: already implemented
    }
}
