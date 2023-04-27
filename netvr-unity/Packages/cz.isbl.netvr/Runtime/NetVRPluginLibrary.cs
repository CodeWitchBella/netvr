using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

namespace Isbl.NetVR
{


    class NetVRPluginLibrary : IDisposable
    {
        const string LibraryName = "netvr_plugin";

        private readonly IsblDynamicLibrary _l;

        public delegate void Logger_Delegate(Int32 level, [MarshalAs(UnmanagedType.LPStr)] string message, [MarshalAs(UnmanagedType.LPStr)] string stack);
        public delegate void SetLogger_Delegate(Logger_Delegate logger);
        private readonly SetLogger_Delegate SetLogger;

        public delegate IntPtr HookGetInstanceProcAddr_Delegate(IntPtr func, bool manualUnhook);
        public readonly HookGetInstanceProcAddr_Delegate HookGetInstanceProcAddr;

        public delegate void Unhook_Delegate();
        public readonly Unhook_Delegate Unhook;

        internal readonly Binary.RPC.GetFn_Delegate GetFn;
        // ADD_FUNC: add delegate and public field above this line


        public readonly Binary.RPC.Cleanup_Delegate Cleanup;


#if !UNITY_EDITOR_WIN
        [DllImport(LibraryName, EntryPoint = "netvr_set_logger")]
        static extern void SetLogger_Native(Logger_Delegate logger);

        [DllImport(LibraryName, EntryPoint = "netvr_hook_get_instance_proc_addr")]
        static extern IntPtr HookGetInstanceProcAddr_Native(IntPtr func, bool manualUnhook);

        [DllImport(LibraryName, EntryPoint = "netvr_unhook")]
        static extern void Unhook_Native();


        [DllImport(LibraryName, EntryPoint = "netvr_cleanup")]
        static extern unsafe void Cleanup_Native(System.UInt32 length, byte* data);

        [DllImport(LibraryName, EntryPoint = "netvr_get_fn")]
        static extern void GetFn_Native([MarshalAs(UnmanagedType.LPStr)] string name, out Binary.RPC.BincodeABI_Delegate fn);
        // ADD_FUNC: add static extern above this line
#endif // !UNITY_EDITOR_WIN

        public NetVRPluginLibrary(Logger_Delegate logger)
        {
            this._l = new IsblDynamicLibrary(LibraryName, "../netvr-rust/target/debug/");
#if UNITY_EDITOR_WIN
            // get function pointers converted to delegates
            _l.GetDelegate("netvr_set_logger", out SetLogger);
            SetLogger(logger);
            _l.GetDelegate("netvr_hook_get_instance_proc_addr", out HookGetInstanceProcAddr);
            _l.GetDelegate("netvr_unhook", out Unhook);
            _l.GetDelegate("netvr_cleanup", out Cleanup);
            _l.GetDelegate("netvr_get_fn", out GetFn);
            // ADD_FUNC: add GetDelegate call above this line
#else
            SetLogger = SetLogger_Native;
            SetLogger(logger);
            HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
            Unhook = Unhook_Native;
            unsafe
            {
                Cleanup = Cleanup_Native;
            };
            GetFn = GetFn_Native;
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
