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

        public static int WriteTo(int data, Span<byte> target, int offset)
        {
            if (target.Length >= offset + 4)
            { BitConverter.TryWriteBytes(target[offset..], data); }
            else if (target.Length > 0)
            { throw new Exception("Target is too smol."); }
            return offset + 4;
        }

        public static int WriteTo(Vector3 data, Span<byte> target, int offset)
        {
            offset = WriteTo(data.x, target, offset);
            offset = WriteTo(data.y, target, offset);
            offset = WriteTo(data.z, target, offset);
            return offset;
        }

        public static int WriteTo(NetDeviceData data, Span<byte> target, int offset)
        {
            offset = WriteTo(data.Position, target, offset);
            offset = WriteTo(data.Rotation, target, offset);
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
    }

    public struct NetIncomingTCPMessage
    {
        [JsonProperty("action")]
        public string Action;

        [JsonProperty("intValue")]
        public int IntValue;
    }
}
