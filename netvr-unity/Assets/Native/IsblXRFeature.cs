using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.XR.OpenXR;
using UnityEngine.XR.OpenXR.Features;

#if UNITY_EDITOR
[UnityEditor.XR.OpenXR.Features.OpenXRFeature(UiName = "Isbl NetVR XR Feature",
    BuildTargetGroups = new[] { UnityEditor.BuildTargetGroup.Standalone, UnityEditor.BuildTargetGroup.Android },
    Company = "Isbl",
    Desc = "Noop extension.",
    // https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_EXT_hand_tracking
    OpenxrExtensionStrings = ExtHandTracking,
    Version = "0.0.1",
    FeatureId = FeatureId)]
#endif
public class IsblXRFeature : OpenXRFeature
{
    public const string ExtHandTracking = "XR_EXT_hand_tracking";
    /// <summary>
    /// The feature id string. This is used to give the feature a well known id for reference.
    /// </summary>
    public const string FeatureId = "cz.isbl.netvr";

    ulong _xrInstance;
    IsblDynamicLibrary _lib;

    protected override bool OnInstanceCreate(ulong xrInstance)
    {
        Debug.Log("OnInstanceCreate");
        _xrInstance = xrInstance;
        _lib?.Dispose();
        _lib = new();
        return true;
    }

    protected override void OnInstanceDestroy(ulong xrInstance)
    {
        Debug.Log("OnInstanceDestroy");
        _lib?.Dispose();
        _xrInstance = 0;
    }

    protected override void OnSystemChange(ulong xrSystem)
    {
        try
        {
            if (OpenXRRuntime.IsExtensionEnabled(ExtHandTracking))
            {
                var status = _lib.OnSystemChange(xrSystem, _xrInstance, xrGetInstanceProcAddr);
                Debug.Log($"_onSystemChange: {status}");
            }
        }
        catch (Exception e)
        {
            Debug.LogError(e);
        }
    }

    delegate int XrGetInstanceProcAddr(ulong instance, [MarshalAs(UnmanagedType.LPStr)] string name, ref IntPtr function);

    static XrGetInstanceProcAddr _xrGetInstanceProcAddrOriginal;
    readonly XrGetInstanceProcAddr _logGetInstanceProcAddr = (ulong instance, string name, ref IntPtr fn) =>
    {
        var result = _xrGetInstanceProcAddrOriginal(instance, name, ref fn);
        Debug.Log($"xrGetInstanceProcAddr({instance}, {name}, ref fn) -> {result}");
        return result;
    };

    protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
    {
        _xrGetInstanceProcAddrOriginal = Marshal.GetDelegateForFunctionPointer<XrGetInstanceProcAddr>(func);

        return Marshal.GetFunctionPointerForDelegate(_logGetInstanceProcAddr);
    }

    protected override void OnSessionCreate(ulong xrSession)
    {
    }
}
