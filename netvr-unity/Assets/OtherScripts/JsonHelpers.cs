using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using UnityEngine;

namespace Isbl.Json
{
    public class Vector3Converter : JsonConverter<Vector3>
    {
        public static Vector3 StaticRead(ref Utf8JsonReader reader)
            => JsonSerializer.Deserialize<Vector3>(ref reader);

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
