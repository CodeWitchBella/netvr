using System;
using Newtonsoft.Json;
using UnityEngine;
using System.Collections.Generic;
using System.Linq;

namespace Isbl
{
    public struct NetDeviceData
    {
        public Vector3 Position, Rotation;
    }

    public class NetStateData
    {
        public bool Initialized;
        public int Id;
        public string IdToken;
        public NetDeviceData Head;
        public IsblStaticXRDevice Left, Right;

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
            Convert(ref offset, ref parsed.Position, binary, toBinary);
            Convert(ref offset, ref parsed.Rotation, binary, toBinary);
        }

        public static void Convert(ref int offset, ref IsblStaticXRDevice parsed, Span<byte> binary, bool toBinary)
        {
            const int Length = IsblStaticXRDevice.DataLength;
            if (binary.Length == 0)
            {
                offset += Length;
                return;
            }

            if (binary.Length >= offset + Length)
            {
                if (toBinary)
                {
                    if (parsed == null)
                        for (int i = 0; i < Length; ++i) binary[i + offset] = 0;
                    else
                        for (int i = 0; i < Length; ++i) binary[i + offset] = parsed.Data[i];
                }
                else
                {
                    if (parsed == null) parsed = new();
                    for (int i = 0; i < Length; ++i)
                        parsed.Data[i] = binary[i + offset];
                }
            }
            else
            {
                throw new Exception("Target is too smol.");
            }

            offset += Length;
        }

        public static void Convert(ref int offset, ref NetStateData parsed, Span<byte> binary, bool toBinary)
        {
            Convert(ref offset, ref parsed.Id, binary, toBinary);
            Convert(ref offset, ref parsed.Head, binary, toBinary);
            Convert(ref offset, ref parsed.Left, binary, toBinary);
            Convert(ref offset, ref parsed.Right, binary, toBinary);
        }

        public static void Convert(ref int offset, Dictionary<int, NetStateData> parsed, Span<byte> binary, bool toBinary)
        {
            int count = parsed.Count;
            Convert(ref offset, ref count, binary, toBinary);

            if (toBinary)
            {
                foreach (var key in parsed.Keys)
                {
                    NetStateData value = parsed[key];
                    Convert(ref offset, ref value, binary, toBinary);
                }
            }
            else
            {
                foreach (var key in parsed.Keys)
                { parsed[key].Initialized = false; }

                for (int i = 0; i < count; ++i)
                {
                    var id = 0;
                    var offsetCopy = offset;
                    Convert(ref offsetCopy, ref id, binary, toBinary);

                    NetStateData value = parsed[id] ?? new();
                    Convert(ref offset, ref value, binary, toBinary);
                    value.Initialized = true;
                }

                foreach (var i in parsed.Where(d => !d.Value.Initialized).ToList())
                { parsed.Remove(i.Key); }
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
