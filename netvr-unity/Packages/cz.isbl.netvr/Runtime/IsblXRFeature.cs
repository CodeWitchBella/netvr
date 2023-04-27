using System;
using System.Collections.Generic;
using System.Timers;
using System.Threading;
using System.IO;
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
        internal NetVRPluginLibrary Lib { get; private set; }
        internal Binary.RPC RPC { get; private set; }


        const bool InstantLog = false;

        static string PluginDirectory()
        {
            if (Application.platform == RuntimePlatform.Android) return Path.Combine(Application.persistentDataPath, "netvr");
            return Path.Combine(System.IO.Directory.GetCurrentDirectory(), "netvr");
        }

        static string _log = "";
        static System.Timers.Timer _pluginTimer;
        [AOT.MonoPInvokeCallback(typeof(NetVRPluginLibrary.Logger_Delegate))]
        static void PluginLogger(Int32 level, string value, string stack)
        {
#if UNITY_EDITOR
        if (stack != "")
            value = value + "\nstack:\n" + stack;
#endif
            if (level <= 2 /* Info or Trace */)
            {
                _log += "\n" + (level == 1 ? "[trace] " : "[info] ") + value.Replace("\n", "\n  ");
                if (_pluginTimer == null)
                {
                    _pluginTimer = new(1000);
                    _pluginTimer.Elapsed += (source, evt) => InfoLogProcess();
                    _pluginTimer.Enabled = true;
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
                    Utils.LogWarning($"[netvr][warn] {value}");
                }
                else /* Error */
                {
                    var levelText = level == 4 ? "error" : level == 5 ? "panic" : "unknown";
                    Utils.LogError($"[netvr][{levelText}] {value}");
                }
            }
        }

        static void InfoLogProcess()
        {
            if (_pluginTimer == null) return;
            Utils.Log($"[netvr] {_log}");
            _log = "";
            _pluginTimer.Dispose();
            _pluginTimer = null;
        }

        private static string GetDataDirectory()
        {
            if (Application.platform == RuntimePlatform.Android) return Path.Combine(Application.persistentDataPath, "netvr");
            return Path.Combine(System.IO.Directory.GetCurrentDirectory(), "netvr");
        }

        protected override bool OnInstanceCreate(ulong xrInstance)
        {
            Utils.Log("OnInstanceCreate");
            XrInstance = xrInstance;

            if (Lib == null)
            {
                Lib = new(PluginLogger);
                if (Lib.GetFn != null) RPC = new(Lib.GetFn, Lib.Cleanup);
            }

            return true;
        }

        protected override void OnSessionBegin(ulong xrSession)
        {
            XrSession = xrSession;
            var dir = GetDataDirectory();
            try { if (!Directory.Exists(dir)) Directory.CreateDirectory(dir); } catch (Exception e) { Utils.LogError($"Failed to create data directory: {e}"); }
            RPC?.Start(new(XrInstance, XrSession, dir));
        }

        protected override void OnSessionEnd(ulong xrSession)
        {
            XrSession = 0;
        }


        protected override void OnInstanceDestroy(ulong xrInstance)
        {
            Utils.Log("OnInstanceDestroy");


            if (NetVRPluginLibrary.DoesUnload) Lib.Unhook();
            Lib?.Dispose();
            Lib = null;
            RPC = null;
            XrInstance = 0;
        }

        protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
        {
            if (Lib == null)
            {
                Lib = new(PluginLogger);
                if (Lib.GetFn != null) RPC = new(Lib.GetFn, Lib.Cleanup);
            }
            return Lib.HookGetInstanceProcAddr(func, manualUnhook: NetVRPluginLibrary.DoesUnload);
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
