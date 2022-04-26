using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class IsblNetRemoteClient : MonoBehaviour
{
    public readonly Dictionary<UInt16, IsblNetRemoteDevice> Devices = new();

#if UNITY_EDITOR
    public ushort Id { get; set; }

    /// <summary>
    /// Used from IsblNetClientDrawer to draw info here.
    /// </summary>
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SelfProperty]
    public GameObject Hack;
#endif
}
