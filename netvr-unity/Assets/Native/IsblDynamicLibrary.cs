using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.XR.OpenXR;
using UnityEngine.XR.OpenXR.Features;

/// <summary>
/// Wrapper around my native dynamic library managing its loading and unloading
/// (when supported)
/// </summary>
class IsblDynamicLibrary : IDisposable
{
    const string LibraryName = "isbl_netvr";

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

    public delegate int Isbl_OnSystemChange_Delegate(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
    public readonly Isbl_OnSystemChange_Delegate OnSystemChange;

#if !UNITY_EDITOR_WIN
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_on_system_change")]
    static extern int Isbl_OnSystemChange_Native(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
#endif // !UNITY_EDITOR_WIN

    public IsblDynamicLibrary()
    {
#if UNITY_EDITOR_WIN
        // copy to new file so that original is still writeable
        const string Prefix = "Assets/Plugins/Windows/x64/";
        File.Copy(Prefix + LibraryName + ".dll", Prefix + LibraryName + "0.dll", true);
        // load
        _library = SystemLibrary.LoadLibrary(Prefix + LibraryName + "0.dll");
        // get function pointers converted to delegates
        SystemLibrary.GetDelegate(_library, "isbl_netvr_on_system_change", out OnSystemChange);
#else
        _onSystemChange = Isbl_OnSystemChange_Native;
#endif
    }

    public void Dispose()
    {
#if UNITY_EDITOR_WIN
        if (_library != default)
        {
            SystemLibrary.FreeLibrary(_library);
            _library = default;
        }
#endif
    }
}
