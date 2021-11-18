using System;
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
        public bool DeviceInfoChanged
        {
            get => Left.DeviceInfoChanged || Right.DeviceInfoChanged;
            set
            {
                Left.DeviceInfoChanged = value;
                Right.DeviceInfoChanged = value;
            }
        }

        static int _byteLength;
        public static int ByteLength
        {
            get
            {
                if (_byteLength == 0)
                {
                    NetStateData data = new();
                    NetData.Convert(ref _byteLength, data, new Span<byte>(), false);
                }
                return _byteLength;
            }
        }

        public void UpdateFrom(ref int offset, Span<byte> binary)
        {
            NetData.Convert(ref offset, this, binary, toBinary: false);
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

        public static void Convert(ref int offset, NetStateData parsed, Span<byte> binary, bool toBinary)
        {
            Convert(ref offset, ref parsed.Id, binary, toBinary);
            Convert(ref offset, ref parsed.Head, binary, toBinary);
            Convert(ref offset, ref parsed.Left, binary, toBinary);
            Convert(ref offset, ref parsed.Right, binary, toBinary);
        }
    }
}
