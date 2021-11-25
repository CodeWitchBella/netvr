using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

namespace Isbl
{
    public class NetStateData
    {
        public bool Initialized;
        public int Id;
        public string IdToken;
        public List<IsblStaticXRDevice> Devices = new();
        public void ResizeDevices(int length)
        {
            while (Devices.Count > length) Devices.RemoveAt(Devices.Count - 1);
            while (Devices.Count < length) Devices.Add(new());
        }
        public bool DeviceInfoChanged
        {
            get => Devices.Exists(d => d.DeviceInfoChanged);
            set { foreach (var d in Devices) d.DeviceInfoChanged = value; }
        }

        public int CalculateSerializationSize()
        {
            return 4 + 4 + 4 * Devices.Count +
                +(from d in Devices select d.CalculateSerializationSize()).Sum();
        }
    }

    public static class NetData
    {
        static void Convert(ref int offset, ref System.Int32 parsed, byte[] binary, bool toBinary)
        {
            if (binary.Length < offset + 4) throw new Exception("Too smol");
            if (toBinary) BinaryPrimitives.WriteInt32LittleEndian(binary[offset..], parsed);
            else parsed = BinaryPrimitives.ReadInt32LittleEndian(binary[offset..]);
            offset += 4;
        }

        static void Convert(ref int offset, IsblStaticXRDevice parsed, byte[] binary, bool toBinary)
        {
            int length = toBinary ? parsed.CalculateSerializationSize() : 0;
            Convert(ref offset, ref length, binary, toBinary);
            if (toBinary)
            {
                var parsedLength = parsed.SerializeData(binary, offset);
                if (parsedLength != length) Debug.LogWarning($"Actual size does not match calculated size. Calculated size: {length}. Actual size: {parsedLength}");
                offset += length;
            }
            else
            {
                var parsedLength = parsed.DeSerializeData(binary, offset);
                if (parsedLength != length) Debug.LogWarning($"Parsed size does not match data-declared size. Declared size: {length}. Parsed size: {parsedLength}.\nThis might happen if some other client added more fields.");
                offset += length;
            }
        }

        public static void Convert(ref int offset, NetStateData parsed, byte[] binary, bool toBinary)
        {
            Convert(ref offset, ref parsed.Id, binary, toBinary);
            int length = parsed.Devices.Count;
            Convert(ref offset, ref length, binary, toBinary);
            parsed.ResizeDevices(length);

            for (int i = 0; i < length; ++i)
                Convert(ref offset, parsed.Devices[i], binary, toBinary);
        }
    }
}
