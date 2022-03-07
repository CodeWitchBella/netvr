using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;
using UnityEngine;

namespace Isbl
{
    public class Vector3JsonConverter : System.Text.Json.Serialization.JsonConverter<Vector3>
    {
        public override Vector3 Read(
            ref Utf8JsonReader reader,
            Type typeToConvert,
            JsonSerializerOptions options) => JsonSerializer.Deserialize<Vector3>(ref reader);

        public override void Write(
            Utf8JsonWriter writer,
            Vector3 value,
            JsonSerializerOptions options)
        {
            if (options.WriteIndented)
            {
                writer.WriteRawValue($"{{ \"x\": {value.x}, \"y\": {value.y}, \"z\": {value.z} }}");
                return;
            }

            writer.WriteStartObject();
            writer.WritePropertyName("x");
            writer.WriteNumberValue(value.x);
            writer.WritePropertyName("y");
            writer.WriteNumberValue(value.y);
            writer.WritePropertyName("z");
            writer.WriteNumberValue(value.z);
            writer.WriteEndObject();
        }
    }

    // NOTE: it must be a class because otherwise Marvin.JsonPatch does not work
    public class NetStateCalibration
    {
        [JsonConverter(typeof(Vector3JsonConverter))]
        [JsonInclude]
        [Newtonsoft.Json.JsonProperty(propertyName: "translate")]
        [JsonPropertyName("translate")]
        public Vector3 Translate;

        [JsonConverter(typeof(Vector3JsonConverter))]
        [JsonInclude]
        [Newtonsoft.Json.JsonProperty(propertyName: "rotate")]
        [JsonPropertyName("rotate")]
        public Vector3 Rotate;

        [JsonConverter(typeof(Vector3JsonConverter))]
        [JsonInclude]
        [Newtonsoft.Json.JsonProperty(propertyName: "scale")]
        [JsonPropertyName("scale")]
        public Vector3 Scale;
    }

    // NOTE: it must be a class because otherwise Marvin.JsonPatch does not work
    public class NetStateClient
    {
        [JsonInclude]
        [Newtonsoft.Json.JsonProperty(propertyName: "connected")]
        [JsonPropertyName("connected")]
        public bool Connected;

        [JsonInclude]
        [Newtonsoft.Json.JsonProperty(propertyName: "calibration")]
        [JsonPropertyName("calibration")]
        public NetStateCalibration Calibration;
    }

    // NOTE: it must be a class because otherwise Marvin.JsonPatch does not work
    public class NetState
    {
        [JsonInclude]
        [Newtonsoft.Json.JsonProperty(propertyName: "clients")]
        [JsonPropertyName("clients")]
        public Dictionary<string, NetStateClient> Clients = new();
    }

    public class NetStateDataOld
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

        public NetStateDataOld(bool local = false)
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

        public static T FromJObjectCamelCase<T>(Newtonsoft.Json.Linq.JObject jobject)
        {
            return jobject.ToObject<T>(new Newtonsoft.Json.JsonSerializer()
            { ContractResolver = new Newtonsoft.Json.Serialization.CamelCasePropertyNamesContractResolver() });
        }

        public static Newtonsoft.Json.Linq.JObject ToJObjectCamelCase<T>(T value)
        {
            return Newtonsoft.Json.Linq.JObject.FromObject(value, new Newtonsoft.Json.JsonSerializer()
            { ContractResolver = new Newtonsoft.Json.Serialization.CamelCasePropertyNamesContractResolver() });
        }
    }
}
