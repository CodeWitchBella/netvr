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


#if UNITY_EDITOR
[UnityEditor.XR.OpenXR.Features.OpenXRFeature(UiName = "Isbl Old NetVR XR Feature",
    BuildTargetGroups = new[] { UnityEditor.BuildTargetGroup.Standalone, UnityEditor.BuildTargetGroup.Android },
    Company = "Isbl",
    Desc = "Noop extension.",
    // https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_EXT_hand_tracking
    OpenxrExtensionStrings = "",//ExtHandTracking,
    Version = "0.0.1",
    FeatureId = FeatureId)]
#endif
public class IsblOldNetXRFeature : OpenXRFeature
{
    public const string ExtHandTracking = "XR_EXT_hand_tracking";
    /// <summary>
    /// The feature id string. This is used to give the feature a well known id for reference.
    /// </summary>
    public const string FeatureId = "cz.isbl.old-netvr";

    ulong _xrInstance = 0;
    internal IsblNetvrLibrary Lib { get; private set; }

    static string _log = "";
    static System.Timers.Timer _timer;
    const bool InstantLog = false;


    [AOT.MonoPInvokeCallback(typeof(IsblNetvrLibrary.Logger_Delegate))]
    static void Logger(string value)
    {
        _log += "\n" + value;
        if (_timer == null)
        {
            _timer = new(1000);
            _timer.Elapsed += (source, evt) =>
            {
                Utils.Log($"From C++:{_log}\nEND");
                _log = "";
                _timer.Dispose();
                _timer = null;
            };
            _timer.Enabled = true;
        }
    }

    static string _logRust = "";
    static System.Timers.Timer _timerRust;
    [AOT.MonoPInvokeCallback(typeof(IsblNetvrLibrary.Logger_Delegate))]
    static void LoggerRust(Int32 level, string value, string stack)
    {
#if UNITY_EDITOR
        if (stack != "")
            value = value + "\nstack:\n" + stack;
#endif
        if (level <= 2 /* Info or Trace */)
        {
            _logRust += "\n" + (level == 2 ? "[trace] " : "[info] ") + value.Replace("\n", "\n  ");
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
        _xrInstance = xrInstance;
        if (Lib == null)
        {
            Lib = new();
            Lib.SetLogger(Logger);
        }

        return true;
    }


    protected override void OnInstanceDestroy(ulong xrInstance)
    {
        Utils.Log("OnInstanceDestroy");

        Lib?.Dispose();
        Lib = null;
        _xrInstance = 0;
    }

    protected override void OnSystemChange(ulong xrSystem)
    {
        try
        {
            if (OpenXRRuntime.IsExtensionEnabled(ExtHandTracking))
            {
                var status = Lib.OnSystemChange(xrSystem, _xrInstance, xrGetInstanceProcAddr);
                Utils.Log($"_onSystemChange: {status}");
            }
        }
        catch (Exception e)
        {
            Utils.LogException(e);
        }
    }

    protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
    {
        if (Lib == null)
        {
            Lib = new();
            Lib.SetLogger(Logger);
        }

        return func;
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