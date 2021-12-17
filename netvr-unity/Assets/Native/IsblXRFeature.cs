using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
#if UNITY_EDITOR
using UnityEditor;
#endif
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

    [AOT.MonoPInvokeCallback(typeof(IsblDynamicLibrary.Logger_Delegate))]
    static void Logger(string value) => Debug.Log($"From C++: {value}");

    protected override bool OnInstanceCreate(ulong xrInstance)
    {
        Debug.Log("OnInstanceCreate");
        _xrInstance = xrInstance;
        if (_lib == null)
        {
            _lib = new();
            _lib.SetLogger(Logger);
        }
        return true;
    }

    protected override void OnInstanceDestroy(ulong xrInstance)
    {
        Debug.Log("OnInstanceDestroy");
        _lib?.Dispose();
        _lib = null;
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

    protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
    {
        _lib?.Dispose();
        _lib = new();
        _lib.SetLogger(Logger);
        return _lib.HookGetInstanceProcAddr(func);
    }

    protected override void OnSessionCreate(ulong xrSession)
    {
    }

#if UNITY_EDITOR
    protected override void GetValidationChecks(List<ValidationRule> results, BuildTargetGroup targetGroup)
    {
        results.Add(new ValidationRule(this)
        {
            message = "IsblNet requires internet permission",
            error = false,
            checkPredicate = () => PlayerSettings.Android.forceInternetPermission,
            fixIt = () => PlayerSettings.Android.forceInternetPermission = true
        });
    }
#endif
}
