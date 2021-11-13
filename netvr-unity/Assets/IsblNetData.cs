using System;
using Newtonsoft.Json;
using UnityEngine;

namespace Isbl
{
    public struct NetDeviceData
    { public Vector3 Position, Rotation; }

    public struct NetStateData
    {
        public int Id;
        public string IdToken;
        public NetDeviceData Head, Left, Right;

        public static readonly int ByteLength = NetData.WriteTo(new NetStateData(), new Span<byte>(), 0);
    }

    public static class NetData
    {
        public static int WriteTo(float data, Span<byte> target, int offset)
        {
            if (target.Length >= offset + 4)
            { BitConverter.TryWriteBytes(target[offset..], data); }
            else if (target.Length > 0)
            { throw new Exception("Target is too smol."); }
            return offset + 4;
        }

        public static int ReadFrom(Span<byte> source, ref float target, int offset)
        {
            if (source.Length >= offset + 4)
            { target = BitConverter.ToSingle(source[offset..]); }
            else
            { throw new Exception("Source is too smol."); }
            return offset + 4;
        }

        public static int WriteTo(int data, Span<byte> target, int offset)
        {
            if (target.Length >= offset + 4)
            { BitConverter.TryWriteBytes(target[offset..], data); }
            else if (target.Length > 0)
            { throw new Exception("Target is too smol."); }
            return offset + 4;
        }

        public static int ReadFrom(Span<byte> source, ref int target, int offset)
        {
            if (source.Length >= offset + 4)
            { target = BitConverter.ToInt32(source[offset..]); }
            else
            { throw new Exception("Source is too smol."); }
            return offset + 4;
        }

        public static int WriteTo(Vector3 data, Span<byte> target, int offset)
        {
            offset = WriteTo(data.x, target, offset);
            offset = WriteTo(data.y, target, offset);
            offset = WriteTo(data.z, target, offset);
            return offset;
        }

        public static int ReadFrom(Span<byte> source, ref Vector3 target, int offset)
        {
            offset = ReadFrom(source, ref target.x, offset);
            offset = ReadFrom(source, ref target.y, offset);
            offset = ReadFrom(source, ref target.z, offset);
            return offset;
        }

        public static int WriteTo(NetDeviceData data, Span<byte> target, int offset)
        {
            offset = WriteTo(data.Position, target, offset);
            offset = WriteTo(data.Rotation, target, offset);
            return offset;
        }

        public static int ReadFrom(Span<byte> source, ref NetDeviceData target, int offset)
        {
            offset = ReadFrom(source, ref target.Position, offset);
            offset = ReadFrom(source, ref target.Rotation, offset);
            return offset;
        }

        public static int WriteTo(NetStateData data, Span<byte> target, int offset)
        {
            offset = WriteTo(data.Id, target, offset);
            offset = WriteTo(data.Head, target, offset);
            offset = WriteTo(data.Left, target, offset);
            offset = WriteTo(data.Right, target, offset);
            return offset;
        }

        public static int ReadFrom(Span<byte> source, ref NetStateData target, int offset)
        {
            offset = ReadFrom(source, ref target.Id, offset);
            offset = ReadFrom(source, ref target.Head, offset);
            offset = ReadFrom(source, ref target.Left, offset);
            offset = ReadFrom(source, ref target.Right, offset);
            return offset;
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
