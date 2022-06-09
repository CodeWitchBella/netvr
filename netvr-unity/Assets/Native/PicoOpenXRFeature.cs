using System;
using System.Collections.Generic;
using System.Timers;
#if UNITY_EDITOR
using UnityEditor;
#endif
using UnityEngine;
using UnityEngine.XR.OpenXR;
using UnityEngine.XR.OpenXR.Features;

#if UNITY_EDITOR
[UnityEditor.XR.OpenXR.Features.OpenXRFeature(UiName = "Pico OpenXR Feature",
    BuildTargetGroups = new[] { UnityEditor.BuildTargetGroup.Android },
    Company = "Isbl",
    Desc = "Fixes PicoVR's mess and enables unity apps to run via OpenXR on their headsets.",
    OpenxrExtensionStrings = "",
    Version = "0.0.1",
    FeatureId = FeatureId)]
#endif
public class PicoOpenXRFeature : OpenXRFeature
{
    /// <summary>
    /// The feature id string. This is used to give the feature a well known id for reference.
    /// </summary>
    public const string FeatureId = "cz.isbl.picoopenxr";

    protected override bool OnInstanceCreate(ulong xrInstance)
    {
        return true;
    }

    protected override void OnInstanceDestroy(ulong xrInstance)
    {

    }

    protected override void OnSystemChange(ulong xrSystem)
    {

    }

    protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
    {
        return func;
    }

    protected override void OnSessionCreate(ulong xrSession)
    {
    }

#if UNITY_EDITOR
    protected override void GetValidationChecks(List<ValidationRule> results, BuildTargetGroup targetGroup)
    {
    }
#endif
}
