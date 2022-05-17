using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

class IsblNetvrLibrary : IDisposable
{
    const string LibraryName = "isbl_netvr";

    private readonly IsblDynamicLibrary _l;

    public delegate int OnSystemChange_Delegate(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
    public readonly OnSystemChange_Delegate OnSystemChange;

    public delegate void Logger_Delegate([MarshalAs(UnmanagedType.LPStr)] string message);
    public delegate void SetLogger_Delegate(Logger_Delegate logger);
    public readonly SetLogger_Delegate SetLogger;

    public delegate IntPtr HookGetInstanceProcAddr_Delegate(IntPtr func);
    public readonly HookGetInstanceProcAddr_Delegate HookGetInstanceProcAddr;

    public delegate int CalibrationCreate_Delegate();
    public readonly CalibrationCreate_Delegate CalibrationCreate;

    public delegate void CalibrationDestroy_Delegate(int handle);
    public readonly CalibrationDestroy_Delegate CalibrationDestroy;

    public delegate void CalibrationAddPair_Delegate(int handle,
        double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
        double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2
    );
    public readonly CalibrationAddPair_Delegate CalibrationAddPair;

    [StructLayout(LayoutKind.Sequential)]
    public struct CalibrationComputeResult
    {
        public double X, Y, Z, Rex, Rey, Rez, Rqx, Rqy, Rqz, Rqw;
    }

    public delegate int CalibrationCompute_Delegate(int handle, out CalibrationComputeResult output);
    public readonly CalibrationCompute_Delegate CalibrationCompute;

    // ADD_FUNC: add delegate and public field above this line

#if !UNITY_EDITOR_WIN
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_on_system_change")]
    static extern int OnSystemChange_Native(ulong xrSystem, ulong xrInstance, IntPtr xrGetInstanceProcAddr);
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_set_logger")]
    static extern void SetLogger_Native(Logger_Delegate logger);
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_hook_get_instance_proc_addr")]
    static extern IntPtr HookGetInstanceProcAddr_Native(IntPtr func);
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_calibration_create")]
    static extern int CalibrationCreate_Native();
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_calibration_destroy")]
    static extern void CalibrationDestroy_Native(int handle);
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_calibration_add_pair")]
    static extern void CalibrationAddPair_Native(int handle,
        double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
        double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2
    );
    [DllImport(LibraryName, EntryPoint = "isbl_netvr_calibration_compute")]
    static extern int CalibrationCompute_Native(int handle, out CalibrationComputeResult output);

    // ADD_FUNC: add static extern above this line
#endif // !UNITY_EDITOR_WIN

    public IsblNetvrLibrary()
    {
        this._l = new IsblDynamicLibrary(LibraryName, "Assets/Plugins/Windows/x64/");
#if UNITY_EDITOR_WIN
        // get function pointers converted to delegates
        _l.GetDelegate("isbl_netvr_on_system_change", out OnSystemChange);
        _l.GetDelegate("isbl_netvr_set_logger", out SetLogger);
        _l.GetDelegate("isbl_netvr_hook_get_instance_proc_addr", out HookGetInstanceProcAddr);
        _l.GetDelegate("isbl_netvr_calibration_create", out CalibrationCreate);
        _l.GetDelegate("isbl_netvr_calibration_destroy", out CalibrationDestroy);
        _l.GetDelegate("isbl_netvr_calibration_add_pair", out CalibrationAddPair);
        _l.GetDelegate("isbl_netvr_calibration_compute", out CalibrationCompute);
        // ADD_FUNC: add GetDelegate call above this line
#else
        OnSystemChange = OnSystemChange_Native;
        SetLogger = SetLogger_Native;
        HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
        CalibrationCreate = CalibrationCreate_Native;
        CalibrationDestroy = CalibrationDestroy_Native;
        CalibrationAddPair = CalibrationAddPair_Native;
        CalibrationCompute = CalibrationCompute_Native;
        // ADD_FUNC: add a statement above this line
#endif
    }

    public void Dispose()
    {
        _l.Dispose();
    }
}
