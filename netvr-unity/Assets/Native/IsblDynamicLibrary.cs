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
        [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Ansi)]
        static public extern IntPtr LoadLibrary(string lpFileName);

        [DllImport("kernel32", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        static public extern bool FreeLibrary(IntPtr hModule);

        [DllImport("kernel32", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
        static public extern IntPtr GetProcAddress(IntPtr hModule, string procedureName);

        static public void GetDelegate<TDelegate>(IntPtr lib, string name, out TDelegate val)
        {
            var addr = GetProcAddress(lib, name);
            if (addr == default)
            {
                Debug.LogWarning($"Can't retrieve address of {name}");
                val = default;
            }
            else
            {
                val = Marshal.GetDelegateForFunctionPointer<TDelegate>(addr);
            }
        }
    }
    IntPtr _library;
    const string Prefix = "Assets/Plugins/Windows/x64/";
    const string FullPath = Prefix + LibraryName + "0.dll";
#endif // UNITY_EDITOR_WIN

    public delegate int OnSystemChange_Delegate(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
    public readonly OnSystemChange_Delegate OnSystemChange;

    public delegate void Logger_Delegate([MarshalAs(UnmanagedType.LPStr)] string message);
    public delegate void SetLogger_Delegate(Logger_Delegate logger);
    public readonly SetLogger_Delegate SetLogger;

    public delegate IntPtr HookGetInstanceProcAddr_Delegate(IntPtr func);
    public readonly HookGetInstanceProcAddr_Delegate HookGetInstanceProcAddr;

#if !UNITY_EDITOR_WIN
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_on_system_change")]
    static extern int OnSystemChange_Native(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_set_logger")]
    static extern void SetLogger_Native(Logger_Delegate logger);
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_hook_get_instance_proc_addr")]
    static extern IntPtr HookGetInstanceProcAddr_Native(IntPtr func);
#endif // !UNITY_EDITOR_WIN

    public IsblDynamicLibrary()
    {
#if UNITY_EDITOR_WIN
        // copy to new file so that original is still writeable
        File.Copy(Prefix + LibraryName + ".dll", FullPath, true);
        // load
        _library = SystemLibrary.LoadLibrary(FullPath);
        if (_library == default) Debug.LogWarning($"Failed to load {FullPath}");
        // get function pointers converted to delegates
        SystemLibrary.GetDelegate(_library, "isbl_netvr_on_system_change", out OnSystemChange);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_set_logger", out SetLogger);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_hook_get_instance_proc_addr", out HookGetInstanceProcAddr);
#else
        OnSystemChange = OnSystemChange_Native;
        SetLogger = SetLogger_Native;
        HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
#endif 
    }

    public void Dispose()
    {
#if UNITY_EDITOR_WIN
        if (_library != default)
        {
            Debug.Log("IsblDynamicLibrary.Dispose()");
            SystemLibrary.FreeLibrary(_library);
            _library = default;
            File.Delete(FullPath);
            try { File.Delete(FullPath + ".meta"); } catch (FileNotFoundException) { }
        }
#endif
    }
}
