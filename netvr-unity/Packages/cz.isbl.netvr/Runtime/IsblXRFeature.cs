using System;
using System.Collections.Generic;
using System.Timers;
using System.Threading;
#if UNITY_EDITOR
using UnityEditor;
#endif
using UnityEngine;
using UnityEngine.XR.OpenXR;
using UnityEngine.XR.OpenXR.Features;

namespace Isbl.NetVR
{

#if UNITY_EDITOR
[UnityEditor.XR.OpenXR.Features.OpenXRFeature(UiName = "Isbl NetVR XR Feature",
    BuildTargetGroups = new[] { UnityEditor.BuildTargetGroup.Standalone, UnityEditor.BuildTargetGroup.Android },
    Company = "Isbl",
    Desc = "Noop extension.",
    // https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_EXT_hand_tracking
    OpenxrExtensionStrings = "",//ExtHandTracking,
    Version = "0.0.1",
    FeatureId = FeatureId)]
#endif
    public class IsblXRFeature : OpenXRFeature
    {
        public static IsblXRFeature Instance => OpenXRSettings.Instance.GetFeature<IsblXRFeature>();

        public const string ExtHandTracking = "XR_EXT_hand_tracking";
        /// <summary>
        /// The feature id string. This is used to give the feature a well known id for reference.
        /// </summary>
        public const string FeatureId = "cz.isbl.netvr";

        internal ulong XrInstance { get; private set; }
        internal ulong XrSession { get; private set; }
        internal IsblRustLibrary RustLib { get; private set; }
        internal Binary.RPC RPC { get; private set; }


        static System.Timers.Timer _timer;
        const bool InstantLog = false;

        static string _logRust = "";
        static System.Timers.Timer _timerRust;
        [AOT.MonoPInvokeCallback(typeof(IsblRustLibrary.Logger_Delegate))]
        static void LoggerRust(Int32 level, string value, string stack)
        {
#if UNITY_EDITOR
        if (stack != "")
            value = value + "\nstack:\n" + stack;
#endif
            if (level <= 2 /* Info or Trace */)
            {
                _logRust += "\n" + (level == 1 ? "[trace] " : "[info] ") + value.Replace("\n", "\n  ");
                if (_timerRust == null)
                {
                    _timerRust = new(1000);
                    _timerRust.Elapsed += (source, evt) => InfoLogProcess();
                    _timerRust.Enabled = true;
                }
#pragma warning disable 0162
                if (InstantLog) InfoLogProcess();
#pragma warning restore 0162
            }
            else
            {
                // make sure that logs aren't out of order
                InfoLogProcess();

                if (level == 3 /* Warn */)
                {
                    Utils.LogWarning($"[rust][warn] {value}");
                }
                else /* Error */
                {
                    var levelText = level == 4 ? "error" : level == 5 ? "panic" : "unknown";
                    Utils.LogError($"[rust][{levelText}] {value}");
                }
            }
        }

        static void InfoLogProcess()
        {
            if (_timerRust == null) return;
            Utils.Log($"[rust] {_logRust}");
            _logRust = "";
            _timerRust.Dispose();
            _timerRust = null;
        }

        protected override bool OnInstanceCreate(ulong xrInstance)
        {
            Utils.Log("OnInstanceCreate");
            XrInstance = xrInstance;

            if (RustLib == null)
            {
                RustLib = new(LoggerRust);
                if (RustLib.GetFn != null) RPC = new(RustLib.GetFn, RustLib.Cleanup);
            }

            return true;
        }

        protected override void OnSessionBegin(ulong xrSession)
        {
            XrSession = xrSession;
            RPC?.Start(new(XrInstance, XrSession));
        }

        protected override void OnSessionEnd(ulong xrSession)
        {
            XrSession = 0;
        }


        protected override void OnInstanceDestroy(ulong xrInstance)
        {
            Utils.Log("OnInstanceDestroy");


            if (IsblRustLibrary.DoesUnload) RustLib.Unhook();
            RustLib?.Dispose();
            RustLib = null;
            RPC = null;
            XrInstance = 0;
        }

        protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
        {
            if (RustLib == null)
            {
                RustLib = new(LoggerRust);
                if (RustLib.GetFn != null) RPC = new(RustLib.GetFn, RustLib.Cleanup);
            }
            return RustLib.HookGetInstanceProcAddr(func, manualUnhook: IsblRustLibrary.DoesUnload);
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

} // namespace Isbl.NetVR
