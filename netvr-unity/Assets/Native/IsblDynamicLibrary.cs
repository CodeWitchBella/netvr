using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

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
                Utils.LogWarning($"Can't retrieve address of {name}");
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
    readonly string _fullPath;
#endif // UNITY_EDITOR_WIN

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

    /**
     * Same as File.OpenRead but returns null if file was not found instead of
     * throwing exception.
     */
    static FileStream SafeOpenRead(string path)
    {
        try { return File.OpenRead(path); }
        catch (FileNotFoundException) { return null; }
    }

    /**
     * Compares contents of given files.
     */
    static bool AreFilesIdentical(string path1, string path2)
    {
        using var fs1 = SafeOpenRead(path1);
        using var fs2 = SafeOpenRead(path2);
        if (fs1 == null || fs2 == null) return fs1 == fs2;

        if (fs1.Length != fs2.Length) return false;

        var fs1buf = new BufferedStream(fs1);
        var fs2buf = new BufferedStream(fs2);
        while (true)
        {
            // Read one byte from each file.
            var file1byte = fs1buf.ReadByte();
            var file2byte = fs2buf.ReadByte();
            if (file1byte == -1) return true;
            if (file1byte != file2byte) return false;
        }
    }

    public IsblDynamicLibrary()
    {
#if UNITY_EDITOR_WIN
        for (int i = 0; ; ++i)
        {
            _fullPath = Prefix + LibraryName + $"{i}.dll";
            try
            {
                using (var watch = new IsblStopwatch("AreFilesIdentical:.dll"))
                    if (AreFilesIdentical(Prefix + LibraryName + ".dll", _fullPath)) break;
                // copy to new file so that original is still writeable
                File.Copy(Prefix + LibraryName + ".dll", _fullPath, true);
                break;
            }
            catch (IOException) when (i != 9)
            {
                // retry
            }
            catch (IOException e)
            {
                Utils.LogException(e);
            }
        }

        // load
        Utils.Log($"Loading library from {_fullPath}");
        _library = SystemLibrary.LoadLibrary(_fullPath);
        if (_library == default) Utils.LogWarning($"Failed to load {_fullPath}");
        // get function pointers converted to delegates
        SystemLibrary.GetDelegate(_library, "isbl_netvr_on_system_change", out OnSystemChange);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_set_logger", out SetLogger);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_hook_get_instance_proc_addr", out HookGetInstanceProcAddr);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_calibration_create", out CalibrationCreate);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_calibration_destroy", out CalibrationDestroy);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_calibration_add_pair", out CalibrationAddPair);
        SystemLibrary.GetDelegate(_library, "isbl_netvr_calibration_compute", out CalibrationCompute);
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
#if UNITY_EDITOR_WIN
        if (_library != default)
        {
            Utils.Log("IsblDynamicLibrary.Dispose()");
            SystemLibrary.FreeLibrary(_library);
            _library = default;
            try
            {
                File.Delete(_fullPath);
                File.Delete(_fullPath + ".meta");
            }
            catch (IOException e) { Utils.LogException(e); }
        }
#endif
    }
}
