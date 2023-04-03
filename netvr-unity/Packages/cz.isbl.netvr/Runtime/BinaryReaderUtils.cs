using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

namespace Isbl.NetVR
{
    /**
     * Collection of functions for working with binary data coming from OpenXR.
     * Reads them and converts them to unity's conventions (mostly handedness conversion).
     */
    internal class BinaryReaderOpenXR
    {
        internal static Vector3 ReadPos(BinaryReader reader)
        {
            return new Vector3(reader.ReadSingle(), reader.ReadSingle(), -reader.ReadSingle());
        }

        internal static Quaternion ReadRot(BinaryReader reader)
        {
            var a = reader.ReadSingle();
            var b = reader.ReadSingle();
            var c = reader.ReadSingle();
            var d = reader.ReadSingle();
            return new Quaternion(-a, -b, c, d);
        }
    }
}
