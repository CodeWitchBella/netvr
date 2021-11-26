using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
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
            get => Devices.Exists(d => d.DeviceInfoChanged && d.HasData);
            set { foreach (var d in Devices) if (d.HasData) d.DeviceInfoChanged = value; }
        }

        public int CalculateSerializationSize()
        {
            return 4 /* Int32 client ID */
                + NetData.Count7BitEncodedIntBytes(Devices.Count(d => d.HasData)) /* Device count */
                + (from d in Devices where d.HasData select d.CalculateSerializationSize()).Sum() /* devices array */;
        }
    }

    public static class NetData
    {
        // Copied from: https://github.com/dotnet/runtime/issues/24473#issuecomment-450755980
        public static int Read7BitEncodedInt(BinaryReader reader)
        {
            sbyte b;
            int r = -7, v = 0;
            do
                v |= ((b = reader.ReadSByte()) & 0x7F) << (r += 7);
            while (b < 0);
            return v;
        }

        public static int Read7BitEncodedInt(Span<byte> data, out int value)
        {
            int offset = 0;
            sbyte b;
            value = 0;
            int r = -7;
            do
                value |= ((b = (sbyte)data[offset++]) & 0x7F) << (r += 7);
            while (b < 0);
            return offset;
        }

        public static void Write7BitEncodedInt(BinaryWriter writer, int i)
        {
            do
            {
                var next = i >> 7;
                writer.Write((byte)((next != 0 ? 0x80 : 0) | i));
                i = next;
            } while (i != 0);
        }

        public static int Write7BitEncodedInt(Span<byte> target, int i)
        {
            int offset = 0;
            do
            {
                var next = i >> 7;
                target[offset++] = (byte)((next != 0 ? 0x80 : 0) | i);
                i = next;
            } while (i != 0);
            return offset;
        }

        public static int CountArrayEncodingBytes(int count, int perElement)
        {
            return count * perElement + Count7BitEncodedIntBytes(count);
        }

        public static int Count7BitEncodedIntBytes(int count)
        {
            int r = 1;
            while ((count >>= 7) != 0) r++;
            return r;
        }
    }
}
