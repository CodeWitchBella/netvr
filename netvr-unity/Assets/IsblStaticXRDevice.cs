using System;
using UnityEngine;
using UnityEngine.XR;

public class IsblStaticXRDevice
{
    static class Offsets
    {
        public const int DeviceRotation = 0;
        public const int PointerRotation = 12;
        public const int DeviceAngularVelocity = 24;
        public const int DevicePosition = 36;
        public const int DeviceVelocity = 48;
        public const int PointerAngularVelocity = 60;
        public const int PointerPosition = 72;
        public const int PointerVelocity = 84;
        public const int Primary2DAxis = 96;
        public const int Grip = 104;
        public const int Trigger = 108;
        public const int TrackingState = 112;
        public const int Bool1 = 116;
        public const int Bool2 = 117;
    }

    public const int DataLength = 118;
    public byte[] Data = new byte[DataLength];
    public string Name = "";
    /// <summary>
    /// Denotes whether information stored outside of Data chaned
    /// </summary>
    ///
    /// True means that the data needs to be transmitted to the server.
    public bool DeviceInfoChanged;
    public bool IsLocal { get; private set; }
    public InputDeviceCharacteristics Characteristics;

    public Quaternion DeviceRotation => ReadQuaternion(Offsets.DeviceRotation);
    public Quaternion PointerRotation => ReadQuaternion(Offsets.PointerRotation);
    public Vector3 DeviceAngularVelocity => ReadVector3(Offsets.DeviceAngularVelocity);
    public Vector3 DevicePosition => ReadVector3(Offsets.DevicePosition);
    public Vector3 DeviceVelocity => ReadVector3(Offsets.DeviceVelocity);
    public Vector3 PointerAngularVelocity => ReadVector3(Offsets.PointerAngularVelocity);
    public Vector3 PointerPosition => ReadVector3(Offsets.PointerPosition);
    public Vector3 PointerVelocity => ReadVector3(Offsets.PointerVelocity);
    public Vector2 Primary2DAxis => ReadVector2(Offsets.Primary2DAxis);
    public float Grip => ReadFloat(Offsets.Grip);
    public float Trigger => ReadFloat(Offsets.Trigger);
    public uint TrackingState => ReadUint(Offsets.TrackingState);
    public bool GripButton => ReadBool(Offsets.Bool1, 0);
    public bool IsTracked => ReadBool(Offsets.Bool1, 1);
    public bool MenuButton => ReadBool(Offsets.Bool1, 2);
    public bool Primary2DAxisClick => ReadBool(Offsets.Bool1, 3);
    public bool Primary2DAxisTouch => ReadBool(Offsets.Bool1, 4);
    public bool PrimaryButton => ReadBool(Offsets.Bool1, 5);
    public bool PrimaryTouch => ReadBool(Offsets.Bool1, 6);
    public bool SecondaryButton => ReadBool(Offsets.Bool1, 7);
    public bool SecondaryTouch => ReadBool(Offsets.Bool2, 0);
    public bool SystemButton => ReadBool(Offsets.Bool2, 1);
    public bool TriggerButton => ReadBool(Offsets.Bool2, 2);
    public bool TriggerTouch => ReadBool(Offsets.Bool2, 3);
    public bool IsLeft => ReadBool(Offsets.Bool2, 4);

    float ReadFloat(int offset)
    {
        return BitConverter.ToSingle(Data, offset);
    }

    Vector3 ReadVector3(int offset)
    {
        return new Vector3(ReadFloat(offset), ReadFloat(offset + 4), ReadFloat(offset + 8));
    }

    Quaternion ReadQuaternion(int offset)
    {
        return Quaternion.Euler(ReadVector3(offset));
    }

    Vector2 ReadVector2(int offset)
    {
        return new Vector2(ReadFloat(offset), ReadFloat(offset + 4));
    }

    uint ReadUint(int offset)
    {
        return BitConverter.ToUInt32(Data, offset);
    }

    bool ReadBool(int offset, int bit)
    {
        return (Data[offset] & (1 << bit)) != 0;
    }

    public void UpdateFromDevice(IsblXRDevice device)
    {
        IsLocal = true;
        if (device == null)
        {
            for (int i = 0; i < Data.Length; ++i) Data[i] = 0;
            if (Name.Length != 0 || Characteristics != 0)
            {
                Name = "";
                Characteristics = 0;
                DeviceInfoChanged = true;
            }
            return;
        }

        if (Characteristics != device.Device.characteristics || Name != device.Device.name)
        {
            Characteristics = device.Device.characteristics;
            Name = device.Device.name;
            DeviceInfoChanged = true;
        }
        UpdateFromDevice(Offsets.DeviceRotation, device.DeviceRotation); // Quaternion
        UpdateFromDevice(Offsets.PointerRotation, device.PointerRotation); // Quaternion
        UpdateFromDevice(Offsets.DeviceAngularVelocity, device.DeviceAngularVelocity); // Vector3
        UpdateFromDevice(Offsets.DevicePosition, device.DevicePosition); // Vector3
        UpdateFromDevice(Offsets.DeviceVelocity, device.DeviceVelocity); // Vector3
        UpdateFromDevice(Offsets.PointerAngularVelocity, device.PointerAngularVelocity); // Vector3
        UpdateFromDevice(Offsets.PointerPosition, device.PointerPosition); // Vector3
        UpdateFromDevice(Offsets.PointerVelocity, device.PointerVelocity); // Vector3
        UpdateFromDevice(Offsets.Primary2DAxis, device.Primary2DAxis); // Vector2
        UpdateFromDevice(Offsets.Grip, device.Grip); // float
        UpdateFromDevice(Offsets.Trigger, device.Trigger); // float
        UpdateFromDevice(Offsets.TrackingState, device.TrackingState); // uint
        UpdateFromDevice(Offsets.Bool1,
            device.GripButton, device.IsTracked, device.MenuButton, device.Primary2DAxisClick,
            device.Primary2DAxisTouch, device.PrimaryButton, device.PrimaryTouch, device.SecondaryButton); // 8 bools
        UpdateFromDevice(Offsets.Bool2,
            device.SecondaryTouch, device.SystemButton, device.TriggerButton, device.TriggerTouch,
            false, false, false, false); // 8 bools
    }

    void UpdateFromDevice(int offset, Quaternion data)
    {
        UpdateFromDevice(offset, data.eulerAngles);
    }

    void UpdateFromDevice(int offset, Vector3 data)
    {
        UpdateFromDevice(offset, data.x);
        UpdateFromDevice(offset + 4, data.y);
        UpdateFromDevice(offset + 8, data.z);
    }

    void UpdateFromDevice(int offset, Vector2 data)
    {
        UpdateFromDevice(offset, data.x);
        UpdateFromDevice(offset + 4, data.y);
    }

    void UpdateFromDevice(int offset, float data)
    {
        if (!BitConverter.TryWriteBytes(Data.AsSpan()[offset..], data))
        {
            Debug.LogWarning($"Failed to write float {data} at offset {offset}");
        }
    }

    void UpdateFromDevice(int offset, uint data)
    {
        BitConverter.TryWriteBytes(Data.AsSpan()[offset..], data);
    }

    void UpdateFromDevice(int offset, bool d0, bool d1, bool d2, bool d3, bool d4, bool d5, bool d6, bool d7)
    {
        static byte C(bool v, int i)
        {
            return (byte)((v ? 1 : 0) << i);
        }
        Data[offset] = (byte)(C(d0, 0) | C(d1, 1) | C(d2, 2) | C(d3, 3) | C(d4, 4) | C(d5, 5) | C(d6, 6) | C(d7, 7));
    }
}
