using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using UnityEngine;

namespace Isbl
{
    /**
     * Contains everything that is directly synchronized with server using json
     * patches. Synchronizations in this structure are mostly automated, but
     * might be too slow for realtime. Therefore it mostly stores configuration.
     *
     * Contains the whole state from server including a copy of local state.
     */
    public class NetServerState
    {
        public struct Calibration
        {
            [JsonConverter(typeof(Json.Vector3Converter))]
            [JsonInclude]
            [JsonPropertyName("translate")]
            public Vector3 Translate;

            [JsonConverter(typeof(Json.QuaternionAsEulerConverter))]
            [JsonInclude]
            [JsonPropertyName("rotate")]
            public Quaternion Rotate;

            [JsonConverter(typeof(Json.Vector3Converter))]
            [JsonInclude]
            [JsonPropertyName("scale")]
            public Vector3 Scale;
        }

        public struct Client
        {
            [JsonInclude]
            [JsonPropertyName("connected")]
            public bool Connected;

            [JsonInclude]
            [JsonPropertyName("calibration")]
            public Calibration Calibration;
        }

        [JsonInclude]
        [JsonPropertyName("clients")]
        public Dictionary<UInt16, Client> Clients = new();
    }

    /**
     * Contains remote information which is synchronized via fast channels and
     * therefore not via the JSON. Contains mostly data from remote devices.
     */
    public class NetFastState
    {
        public struct RemoteDevice
        {
            public IsblNetRemoteDevice Device;
            public IsblStaticXRDevice DeviceData;
        }
        public readonly Dictionary<UInt16, Dictionary<UInt16, RemoteDevice>> Clients = new();

        public bool TryGetRemoteDevice(UInt16 clientId, UInt16 deviceId, out RemoteDevice outDevice)
        {
            if (Clients.TryGetValue(clientId, out var devices))
                return devices.TryGetValue(deviceId, out outDevice);
            outDevice = default;
            return false;
        }
    }

    public static class NetUtils
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

        public static UInt16 CastUInt16(int value)
        {
            if (value < 0 || value > 65535) throw new Exception("Expected value storeable in UInt16");
            return (UInt16)value;
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
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
                IncludeFields = true,
            }));
        }
        public static JsonElement JsonFromObject(object value)
        {
            var node = JsonDocument.Parse(JsonSerializer.Serialize(value));
            return node.RootElement;
        }
    }
}
