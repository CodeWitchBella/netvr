using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

namespace Isbl.NetVR
{


    class IsblRustLibrary : IDisposable
    {
        const string LibraryName = "isbl_netvr_rust";

        private readonly IsblDynamicLibrary _l;

        public delegate void Logger_Delegate(Int32 level, [MarshalAs(UnmanagedType.LPStr)] string message, [MarshalAs(UnmanagedType.LPStr)] string stack);
        public delegate void SetLogger_Delegate(Logger_Delegate logger);
        public readonly SetLogger_Delegate SetLogger;

        public delegate IntPtr HookGetInstanceProcAddr_Delegate(IntPtr func, bool manualUnhook);
        public readonly HookGetInstanceProcAddr_Delegate HookGetInstanceProcAddr;

        public delegate void Unhook_Delegate();
        public readonly Unhook_Delegate Unhook;

        public delegate void Tick_Delegate(ulong xrInstance);
        public readonly Tick_Delegate Tick;
        // ADD_FUNC: add delegate and public field above this line

#if !UNITY_EDITOR_WIN
        [DllImport(LibraryName, EntryPoint = "netvr_set_logger")]
        static extern void SetLogger_Native(Logger_Delegate logger);

        [DllImport(LibraryName, EntryPoint = "netvr_hook_get_instance_proc_addr")]
        static extern IntPtr HookGetInstanceProcAddr_Native(IntPtr func, bool manualUnhook);

        [DllImport(LibraryName, EntryPoint = "netvr_unhook")]
        static extern void Unhook_Native();

        [DllImport(LibraryName, EntryPoint = "netvr_tick")]
        static extern void Tick_Native(ulong xrInstance);
        // ADD_FUNC: add static extern above this line
#endif // !UNITY_EDITOR_WIN

        public IsblRustLibrary()
        {
            this._l = new IsblDynamicLibrary(LibraryName, "../netvr-rust/target/debug/");
#if UNITY_EDITOR_WIN
    // get function pointers converted to delegates
    _l.GetDelegate("netvr_set_logger", out SetLogger);
    _l.GetDelegate("netvr_hook_get_instance_proc_addr", out HookGetInstanceProcAddr);
    _l.GetDelegate("netvr_unhook", out Unhook);
    _l.GetDelegate("netvr_tick", out Tick);
    // ADD_FUNC: add GetDelegate call above this line
#else
            SetLogger = SetLogger_Native;
            HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
            Unhook = Unhook_Native;
            Tick = Tick_Native;
            // ADD_FUNC: add a statement above this line
#endif
        }

        public void Dispose()
        {
            _l.Dispose();
        }

        public const bool DoesUnload = IsblDynamicLibrary.DoesUnload;
    }

}
