using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

namespace Isbl.NetVR
{
    /**
     * Collection of functions for working with binary data coming from the plugin.
     * Reads them and converts them to unity's conventions (mostly handedness conversion).
     */
    internal class Convertor
    {
        internal static Vector3 Vector3(Isbl.NetVR.Binary.Vec3 v)
        {
            return new Vector3(v.x, v.y, -v.z);
            //return new Vector3(-v.x, v.y, v.z);
        }

        internal static Quaternion Quaternion(Isbl.NetVR.Binary.Quaternion v)
        {
            return new Quaternion(-v.x, -v.y, v.z, v.w);
        }
    }
}
