using System;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using UnityEngine;

namespace Isbl.Json
{
    public class Vector3Converter : JsonConverter<Vector3>
    {
        private static readonly byte[] _propX = Encoding.UTF8.GetBytes("x");
        private static readonly byte[] _propY = Encoding.UTF8.GetBytes("y");
        private static readonly byte[] _propZ = Encoding.UTF8.GetBytes("z");

        private static float ReadFloat(ref Utf8JsonReader reader)
        {
            if (!reader.Read()) throw new Exception("Unexpected end");
            return (float)reader.GetDouble();
        }

        public static Vector3 StaticRead(ref Utf8JsonReader reader)
        {
            if (reader.TokenType != JsonTokenType.StartObject) throw new Exception("Expected object start");

            Vector3 result = Vector3.zero;
            while (reader.Read())
            {
                switch (reader.TokenType)
                {
                case JsonTokenType.EndObject: return result;
                case JsonTokenType.PropertyName:
                    if (reader.ValueTextEquals(_propX)) result.x = ReadFloat(ref reader);
                    else if (reader.ValueTextEquals(_propY)) result.y = ReadFloat(ref reader);
                    else if (reader.ValueTextEquals(_propZ)) result.z = ReadFloat(ref reader);
                    else reader.Skip();
                    break;
                default: throw new Exception("Unexpected token when parsing vector");
                }
            }
            throw new Exception("Unexpected end");
        }

        public override Vector3 Read(
            ref Utf8JsonReader reader,
            Type typeToConvert,
            JsonSerializerOptions options) => StaticRead(ref reader);

        public static void StaticWrite(Utf8JsonWriter writer,
            Vector3 value,
            JsonSerializerOptions options)
        {
#if NET6_0
            if (options.WriteIndented)
            {
                // print on one line even when WriteIndented is enabled
                writer.WriteRawValue($"{{ \"x\": {value.x}, \"y\": {value.y}, \"z\": {value.z} }}");
                return;
            }
#endif

            writer.WriteStartObject();
            writer.WritePropertyName("x");
            writer.WriteNumberValue(value.x);
            writer.WritePropertyName("y");
            writer.WriteNumberValue(value.y);
            writer.WritePropertyName("z");
            writer.WriteNumberValue(value.z);
            writer.WriteEndObject();
        }

        public override void Write(
            Utf8JsonWriter writer,
            Vector3 value,
            JsonSerializerOptions options)
        {
            StaticWrite(writer, value, options);
        }
    }

    public class QuaternionAsEulerConverter : JsonConverter<Quaternion>
    {
        public override Quaternion Read(
            ref Utf8JsonReader reader,
            Type typeToConvert,
            JsonSerializerOptions options)
        {
            var angles = Vector3Converter.StaticRead(ref reader);
            return Quaternion.Euler(angles * (180f / MathF.PI));
        }

        public override void Write(
            Utf8JsonWriter writer,
            Quaternion value,
            JsonSerializerOptions options)
        {
            Vector3Converter.StaticWrite(writer, value.eulerAngles * (MathF.PI / 180f), options);
        }
    }
}
