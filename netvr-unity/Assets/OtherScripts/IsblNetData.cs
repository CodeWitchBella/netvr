using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Newtonsoft.Json.Linq;
using Newtonsoft.Json.Serialization;
using UnityEngine;

namespace Isbl
{
    public class NetStateData
    {
        public bool Initialized;
        public int Id;
        public string IdToken;

        public Vector3 CalibrationPosition = Vector3.zero;
        public Quaternion CalibrationRotation = Quaternion.identity;
        public Vector3 CalibrationScale = Vector3.one;

        public readonly Dictionary<int, IsblStaticXRDevice> Devices = new();
        public readonly Dictionary<int, IsblXRDevice> LocalDevices;
        public bool DeviceInfoChanged
        {
            get => Devices.Values.Any(d => d.DeviceInfoChanged && d.HasData);
            set { foreach (var d in Devices) if (d.Value.HasData) d.Value.DeviceInfoChanged = value; }
        }

        public int CalculateSerializationSize()
        {
            return 4 /* Int32 client ID */
                + NetData.Count7BitEncodedIntBytes(Devices.Count(d => d.Value.HasData)) /* Device count */
                + (from d in Devices where d.Value.HasData select d.Value.CalculateSerializationSize()).Sum() /* devices array */;
        }

        public NetStateData(bool local = false)
        {
            if (local) LocalDevices = new();
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

        public static T FromJObjectCamelCase<T>(JObject jobject)
        {
            return jobject.ToObject<T>(new Newtonsoft.Json.JsonSerializer()
            { ContractResolver = new CamelCasePropertyNamesContractResolver() });
        }

        public static JObject ToJObjectCamelCase<T>(T value)
        {
            return JObject.FromObject(value, new Newtonsoft.Json.JsonSerializer()
            { ContractResolver = new CamelCasePropertyNamesContractResolver() });
        }
    }
}
