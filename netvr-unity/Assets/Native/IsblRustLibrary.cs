using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

class IsblRustLibrary : IDisposable
{
    private IsblDynamicLibrary _l;

    public delegate void Logger_Delegate([MarshalAs(UnmanagedType.LPStr)] string message);
    public delegate void SetLogger_Delegate(Logger_Delegate logger);
    public readonly SetLogger_Delegate SetLogger;
    // ADD_FUNC: add delegate and public field above this line

#if !UNITY_EDITOR_WIN
    [DllImport(LibraryName, EntryPoint = "netvr_set_logger")]
    static extern void SetLogger_Native(Logger_Delegate logger);
    // ADD_FUNC: add static extern above this line
#endif // !UNITY_EDITOR_WIN

    public IsblRustLibrary()
    {
        this._l = new IsblDynamicLibrary("isbl_netvr_rust", "../netvr-rust/target/release/");
#if UNITY_EDITOR_WIN
        // get function pointers converted to delegates
        _l.GetDelegate("netvr_set_logger", out SetLogger);
        // ADD_FUNC: add GetDelegate call above this line
#else
        SetLogger = SetLogger_Native;
        // ADD_FUNC: add a statement above this line
#endif
    }

    public void Dispose()
    {
        _l.Dispose();
    }
}
