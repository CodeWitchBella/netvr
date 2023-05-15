using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using UnityEngine;

public static class JsonUtils
{

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
