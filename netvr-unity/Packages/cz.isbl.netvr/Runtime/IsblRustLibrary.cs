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
        public readonly SetLogger_Delegate SetLogger;

        public delegate IntPtr HookGetInstanceProcAddr_Delegate(IntPtr func, bool manualUnhook);
        public readonly HookGetInstanceProcAddr_Delegate HookGetInstanceProcAddr;

        public delegate void Unhook_Delegate();
        public readonly Unhook_Delegate Unhook;

        public delegate void Tick_Delegate(ulong xrInstance);
        public readonly Tick_Delegate Tick;
        // ADD_FUNC: add delegate and public field above this line

        private unsafe delegate System.Int32 BincodeABI_Delegate(ref System.UInt32 length, ref byte* data);
        private unsafe delegate void Cleanup_Delegate(System.UInt32 length, byte* data);
        private readonly Cleanup_Delegate Cleanup;

        private readonly BincodeABI_Delegate Bincode_ReadRemoteDevices;
        // ADD_BINCODE: Add a line above this one
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
        // ADD_FUNC: add static extern above this line

        [DllImport(LibraryName, EntryPoint = "netvr_read_remote_devices")]
        static extern System.Int32 ReadRemoteDevices_Native(ref System.UInt32 length, ref byte* data);
        // ADD_BINCODE: Add a line above this one
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
            _l.GetDelegate("netvr_cleanup", out  Cleanup);
            // ADD_FUNC: add GetDelegate call above this line

            _l.GetDelegate("netvr_read_remote_devices", out Bincode_ReadRemoteDevices);
            // ADD_BINCODE: Add a line above this one
#else
            SetLogger = SetLogger_Native;
            HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
            Unhook = Unhook_Native;
            Tick = Tick_Native;
            Cleanup = Cleanup_Native;
            // ADD_FUNC: add a statement above this line

            Bincode_ReadRemoteDevices = ReadRemoteDevices_Native;
            // ADD_BINCODE: Add a line above this one
#endif
        }

        public void Dispose()
        {
            _l.Dispose();
        }

        private byte[] FunctionCall(byte[] value, BincodeABI_Delegate func)
        {
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

        public Isbl.NetVR.Binary.RemoteDevices ReadRemoteDevices(Isbl.NetVR.Binary.JustInstance input) => Isbl.NetVR.Binary.RemoteDevices.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), Bincode_ReadRemoteDevices));
        // ADD_BINCODE: Add a line above this one

        public const bool DoesUnload = IsblDynamicLibrary.DoesUnload;
    }

}
