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
    public class Vector3JsonConverter : JsonConverter<Vector3>
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
                // print on one line even when WriteIndented is enabled
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

    public struct NetStateCalibration
    {
        [JsonConverter(typeof(Vector3JsonConverter))]
        [JsonInclude]
        [JsonPropertyName("translate")]
        public Vector3 Translate;

        [JsonConverter(typeof(Vector3JsonConverter))]
        [JsonInclude]
        [JsonPropertyName("rotate")]
        public Vector3 Rotate;

        [JsonConverter(typeof(Vector3JsonConverter))]
        [JsonInclude]
        [JsonPropertyName("scale")]
        public Vector3 Scale;
    }

    public struct NetStateClient
    {
        [JsonInclude]
        [JsonPropertyName("connected")]
        public bool Connected;

        [JsonInclude]
        [JsonPropertyName("calibration")]
        public NetStateCalibration Calibration;
    }

    public class NetState
    {
        [JsonInclude]
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

        public static JsonDocument ToJsonCamelCase<T>(T value)
        {
            return JsonDocument.Parse(JsonSerializer.Serialize(value, new JsonSerializerOptions
            {
                DictionaryKeyPolicy = JsonNamingPolicy.CamelCase,
                IncludeFields = true,
            }));
        }

        public static T FromJsonCamelCase<T>(JsonDocument doc)
        {
            return doc.Deserialize<T>(new JsonSerializerOptions
            {
                DictionaryKeyPolicy = JsonNamingPolicy.CamelCase
            });
        }

        public static JsonObject JsonFromObject(object value)
        {
            var node = JsonNode.Parse(JsonSerializer.Serialize(value));
            if (typeof(JsonObject) == node.GetType()) return (JsonObject)node;
            throw new Exception("Failed to convert to json object");
        }
    }
}
