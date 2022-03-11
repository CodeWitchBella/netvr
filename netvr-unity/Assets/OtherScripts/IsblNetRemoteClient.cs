using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class IsblNetRemoteClient : MonoBehaviour
{
    public readonly Dictionary<UInt16, IsblNetRemoteDevice> Devices = new();
}
