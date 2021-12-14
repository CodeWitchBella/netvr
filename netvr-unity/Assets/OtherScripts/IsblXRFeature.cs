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
    const string LibraryName = "isbl_netvr";

    ulong _xrInstance;

    protected override IntPtr HookGetInstanceProcAddr(IntPtr func)
    {
        return func;
    }

#if UNITY_EDITOR_WIN
    /// <summary>
    /// Windows-specific functions for dll loading and unloading so that I dont
    /// have to restart Unity Editor every time I change the dll.
    /// </summary>
    static class SystemLibrary
    {
        [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
        static public extern IntPtr LoadLibrary(string lpFileName);

        [DllImport("kernel32", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        static public extern bool FreeLibrary(IntPtr hModule);

        [DllImport("kernel32")]
        static public extern IntPtr GetProcAddress(IntPtr hModule, string procedureName);

        static public void GetDelegate<TDelegate>(IntPtr lib, string name, out TDelegate val)
        {
            val = Marshal.GetDelegateForFunctionPointer<TDelegate>(GetProcAddress(lib, name));
        }
    }
    IntPtr _library;
#endif // UNITY_EDITOR_WIN

    delegate int Isbl_OnSystemChange_Delegate(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
    Isbl_OnSystemChange_Delegate _onSystemChange;

#if !UNITY_EDITOR_WIN
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_on_system_change")]
    static extern int Isbl_OnSystemChange_Native(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
#endif // !UNITY_EDITOR_WIN

    protected override bool OnInstanceCreate(ulong xrInstance)
    {
        Debug.Log("OnInstanceCreate");
        _xrInstance = xrInstance;
#if UNITY_EDITOR_WIN
        // copy to new file so that original is still writeable
        const string Prefix = "Assets/Plugins/Windows/x64/";
        File.Copy(Prefix + LibraryName + ".dll", Prefix + LibraryName + "0.dll", true);
        // load
        _library = SystemLibrary.LoadLibrary(Prefix + LibraryName + "0.dll");
        // get function pointers converted to delegates
        SystemLibrary.GetDelegate(_library, "isbl_netvr_on_system_change", out _onSystemChange);
#else
        _onSystemChange = Isbl_OnSystemChange_Native;
#endif

        return true;
    }

    protected override void OnInstanceDestroy(ulong xrInstance)
    {
        Debug.Log("OnInstanceDestroy");
#if UNITY_EDITOR_WIN
        SystemLibrary.FreeLibrary(_library);
#endif
        _xrInstance = 0;
    }

    protected override void OnSystemChange(ulong xrSystem)
    {
        try
        {
            if (OpenXRRuntime.IsExtensionEnabled(ExtHandTracking))
            {
                var status = _onSystemChange(xrSystem, _xrInstance, xrGetInstanceProcAddr);
                Debug.Log($"_onSystemChange: {status}");
            }
        }
        catch (Exception e)
        {
            Debug.LogError(e);
        }
    }

    protected override void OnSessionCreate(ulong xrSession)
    {
    }
}
