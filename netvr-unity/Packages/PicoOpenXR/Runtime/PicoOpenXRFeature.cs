using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEditor;
using UnityEngine;
using UnityEngine.XR.OpenXR.Features;
using UnityEngine.XR.OpenXR;

#if UNITY_EDITOR
using UnityEngine.XR.OpenXR.Features.Interactions;
#endif

namespace Isbl.PicoOpenXR.PicoOpenXRSupport
{
#if UNITY_EDITOR
    [UnityEditor.XR.OpenXR.Features.OpenXRFeature(UiName = "Pico Neo 3 OpenXR Support",
        Desc = "Necessary to deploy an Pico Neo 3 OpenXR compatible app.",
        Company = "Isbl",
        //DocumentationLink = "https://developer.oculus.com/downloads/package/oculus-openxr-mobile-sdk/",
        OpenxrExtensionStrings = "XR_KHR_loader_init_android XR_PICO_configs_ext XR_PICO_reset_sensor",
        Version = "1.0.0",
        BuildTargetGroups = new[] { BuildTargetGroup.Android },
        CustomRuntimeLoaderBuildTargets = new[] { BuildTarget.Android },
        FeatureId = FeatureId
    )]
#endif
    public class PicoOpenXRFeature : OpenXRFeature
    {
        /// <summary>
        /// The feature id string. This is used to give the feature a well known id for reference.
        /// </summary>
        public const string FeatureId = "cz.isbl.picoopenxr";


        delegate IntPtr XrSetConfigPICO_Delegate(ulong session, int configIndex, [MarshalAs(UnmanagedType.LPStr)] string data);

        delegate int XrGetInstanceProcAddr_Delegate(ulong instance, [MarshalAs(UnmanagedType.LPStr)] string name, out IntPtr function);

        private XrGetInstanceProcAddr_Delegate _getInstanceProcAddr;
        ulong _instance;

        protected override bool OnInstanceCreate(ulong xrInstance)
        {
            if (!OpenXRRuntime.IsExtensionEnabled("XR_PICO_configs_ext") || !OpenXRRuntime.IsExtensionEnabled("XR_PICO_reset_sensor") || !OpenXRRuntime.IsExtensionEnabled("XR_KHR_loader_init_android"))
            {
                Debug.LogWarning("Required extension is not enabled, disabling PICO OpenXR.");

                // Return false here to indicate the system should disable your feature for this execution.  
                // Note that if a feature is marked required, returning false will cause the OpenXRLoader to abort and try another loader.
                return false;
            }

            _instance = xrInstance;
            _getInstanceProcAddr = Marshal.GetDelegateForFunctionPointer<XrGetInstanceProcAddr_Delegate>(xrGetInstanceProcAddr);
            return true;
        }

        public TDelegate GetDelegate<TDelegate>(string name)
        {
            var status = _getInstanceProcAddr(_instance, name, out var addr);
            if (status != 0) throw new Exception($"Expected XrStatus 0 for getInstanceProcAddr of {name}, got {status}");
            return Marshal.GetDelegateForFunctionPointer<TDelegate>(addr);
        }

        protected override void OnSessionBegin(ulong xrSession)
        {
            var setConfigPICO = GetDelegate<XrSetConfigPICO_Delegate>("xrSetConfigPICO");
            setConfigPICO(xrSession, 1/*TRACKING_ORIGIN*/, "1" /*floorLevel*/);
        }


#if UNITY_EDITOR
        protected override void GetValidationChecks(List<ValidationRule> rules, BuildTargetGroup targetGroup)
        {
        }
#endif
    }
}
