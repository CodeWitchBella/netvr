using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

namespace Isbl.NetVR
{


    class IsblRustLibrary : IDisposable
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

        private delegate void GetFn_Delegate([MarshalAs(UnmanagedType.LPStr)] string name, out BincodeABI_Delegate fn);
        private readonly GetFn_Delegate GetFn;

        public delegate void Tick_Delegate(ulong xrInstance);
        public readonly Tick_Delegate Tick;
        // ADD_FUNC: add delegate and public field above this line

        private unsafe delegate System.Int32 BincodeABI_Delegate(ref System.UInt32 length, ref byte* data);
        private unsafe delegate void Cleanup_Delegate(System.UInt32 length, byte* data);
        private readonly Cleanup_Delegate Cleanup;


#if !UNITY_EDITOR_WIN
        [DllImport(LibraryName, EntryPoint = "netvr_set_logger")]
        static extern void SetLogger_Native(Logger_Delegate logger);

        [DllImport(LibraryName, EntryPoint = "netvr_hook_get_instance_proc_addr")]
        static extern IntPtr HookGetInstanceProcAddr_Native(IntPtr func, bool manualUnhook);

        [DllImport(LibraryName, EntryPoint = "netvr_unhook")]
        static extern void Unhook_Native();

        [DllImport(LibraryName, EntryPoint = "netvr_tick")]
        static extern void Tick_Native(ulong xrInstance);

        [DllImport(LibraryName, EntryPoint = "netvr_cleanup")]
        static extern System.UInt32 Cleanup_Native(System.UInt32 length, byte* data);

        [DllImport(LibraryName, EntryPoint = "netvr_get_fn")]
        static extern void GetFn_Native([MarshalAs(UnmanagedType.LPStr)] string name, IntPtr fn);
        // ADD_FUNC: add static extern above this line
#endif // !UNITY_EDITOR_WIN

        public IsblRustLibrary(Logger_Delegate logger)
        {
            this._l = new IsblDynamicLibrary(LibraryName, "../netvr-rust/target/debug/");
#if UNITY_EDITOR_WIN
            // get function pointers converted to delegates
            _l.GetDelegate("netvr_set_logger", out SetLogger);
            SetLogger(logger);
            _l.GetDelegate("netvr_hook_get_instance_proc_addr", out HookGetInstanceProcAddr);
            _l.GetDelegate("netvr_unhook", out Unhook);
            _l.GetDelegate("netvr_tick", out Tick);
            _l.GetDelegate("netvr_cleanup", out Cleanup);
            _l.GetDelegate("netvr_get_fn", out GetFn);
            // ADD_FUNC: add GetDelegate call above this line
#else
            SetLogger = SetLogger_Native;
            SetLogger(logger);
            HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
            Unhook = Unhook_Native;
            Tick = Tick_Native;
            Cleanup = Cleanup_Native;
            GetFn = GetFn_Native;
            // ADD_FUNC: add a statement above this line
#endif
        }

        public void Dispose()
        {
            _l.Dispose();
        }

        private byte[] FunctionCall(byte[] value, ref BincodeABI_Delegate func, string name)
        {
            if (func == null) GetFn(name, out func);
            if (func == null) throw new Exception($"Function {name} not found");

            byte[] bytes = value == null ? null : value;
            unsafe
            {
                fixed (byte* bytes_ptr = bytes)
                {
                    byte* data = bytes_ptr;
                    var byte_count = (UInt32)bytes.Length;
                    var code = func(ref byte_count, ref data);
                    if (code != 0) throw new Exception($"ReadRemoteDeviceData failed with error {code}");
                    if (data == null || data == bytes_ptr) return null;
                    bytes = new byte[byte_count];
                    Marshal.Copy((IntPtr)data, bytes, 0, (Int32)byte_count);
                    Cleanup(byte_count, data);
                };
            }
            return bytes;
        }

        private BincodeABI_Delegate Cache_ReadRemoteDevices;
        public Isbl.NetVR.Binary.RemoteDevices ReadRemoteDevices(Isbl.NetVR.Binary.JustInstance input) => Isbl.NetVR.Binary.RemoteDevices.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_ReadRemoteDevices, "read_remote_devices"));
        // ADD_BINCODE: Duplicate the above two lines below this line and change the relevant parameters

        public const bool DoesUnload = IsblDynamicLibrary.DoesUnload;
    }

}
