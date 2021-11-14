using System;
using Newtonsoft.Json;
using UnityEngine;
using System.Collections.Generic;
using System.Linq;

namespace Isbl
{
    public struct NetDeviceData
    {
        public byte Type;
        public Vector3 Position, Rotation;
    }

    public struct NetStateData
    {
        public int Id;
        public string IdToken;
        public NetDeviceData Head, Left, Right;

        static int _byteLength;
        public static int ByteLength
        {
            get
            {
                if (_byteLength == 0)
                {
                    NetStateData data = new();
                    NetData.Convert(ref _byteLength, ref data, new Span<byte>(), false);
                }
                return _byteLength;
            }
        }
    }

    public static class NetData
    {
        public static void Convert(ref int offset, ref float parsed, Span<byte> binary, bool toBinary)
        {
            if (binary.Length >= offset + 4)
            {
                if (toBinary) BitConverter.TryWriteBytes(binary[offset..], parsed);
                else parsed = BitConverter.ToSingle(binary[offset..]);
            }
            else if (binary.Length > 0)
            { throw new Exception("Target is too smol."); }

            offset += 4;
        }

        public static void Convert(ref int offset, ref int parsed, Span<byte> binary, bool toBinary)
        {
            if (binary.Length >= offset + 4)
            {
                if (toBinary) BitConverter.TryWriteBytes(binary[offset..], parsed);
                else parsed = BitConverter.ToInt32(binary[offset..]);
            }
            else if (binary.Length > 0)
            { throw new Exception("Target is too smol."); }

            offset += 4;
        }

        public static void Convert(ref int offset, ref byte parsed, Span<byte> binary, bool toBinary)
        {
            if (binary.Length >= offset + 1)
            {
                if (toBinary) binary[offset] = parsed;
                else parsed = binary[offset];
            }
            else if (binary.Length > 0)
            { throw new Exception("Target is too smol."); }

            offset++;
        }

        public static void Convert(ref int offset, ref Vector3 parsed, Span<byte> binary, bool toBinary)
        {
            Convert(ref offset, ref parsed.x, binary, toBinary);
            Convert(ref offset, ref parsed.y, binary, toBinary);
            Convert(ref offset, ref parsed.z, binary, toBinary);
        }

        public static void Convert(ref int offset, ref NetDeviceData parsed, Span<byte> binary, bool toBinary)
        {
            Convert(ref offset, ref parsed.Type, binary, toBinary);
            Convert(ref offset, ref parsed.Position, binary, toBinary);
            Convert(ref offset, ref parsed.Rotation, binary, toBinary);
        }

        public static void Convert(ref int offset, ref NetStateData parsed, Span<byte> binary, bool toBinary)
        {
            Convert(ref offset, ref parsed.Id, binary, toBinary);
            Convert(ref offset, ref parsed.Head, binary, toBinary);
            Convert(ref offset, ref parsed.Left, binary, toBinary);
            Convert(ref offset, ref parsed.Right, binary, toBinary);
        }

        public static void Convert(ref int offset, List<NetStateData> parsed, Span<byte> binary, bool toBinary)
        {
            int count = parsed.Count;
            Convert(ref offset, ref count, binary, toBinary);

            if (!toBinary)
            {
                // resize
                if (count < parsed.Count)
                    parsed.RemoveRange(count, parsed.Count - count);
                else if (count > parsed.Capacity) parsed.Capacity = count;

                while (count > parsed.Count)
                    parsed.Add(new());
            }

            for (var i = 0; i < parsed.Count; i++)
            {
                NetStateData value = parsed[i];
                Convert(ref offset, ref value, binary, toBinary);
                parsed[i] = value;
            }
        }

    }

    public struct NetIncomingTCPMessage
    {
        [JsonProperty("action")]
        public string Action;

        [JsonProperty("intValue")]
        public int IntValue;
        [JsonProperty("stringValue")]
        public string StringValue;
    }
}
