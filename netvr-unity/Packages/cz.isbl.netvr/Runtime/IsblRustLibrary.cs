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

        private delegate System.Int32 ReadRemoteDeviceData_Delegate(ulong xrInstance, out System.UInt32 length, out IntPtr data);
        private readonly ReadRemoteDeviceData_Delegate ReadRemoteDeviceData;

        private delegate void Cleanup_Delegate(System.UInt32 length, IntPtr data);
        private readonly Cleanup_Delegate Cleanup;
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

        [DllImport(LibraryName, EntryPoint = "netvr_read_remote_device_data")]
        static extern System.Int32 ReadRemoteDeviceData_Native(ulong xrInstance, out System.UInt32 length, out IntPtr data);

        [DllImport(LibraryName, EntryPoint = "netvr_cleanup")]
        static extern System.UInt32 Cleanup_Native(System.UInt32 length, IntPtr data);
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
            _l.GetDelegate("netvr_read_remote_device_data", out ReadRemoteDeviceData);
            _l.GetDelegate("netvr_cleanup", out  Cleanup);
            // ADD_FUNC: add GetDelegate call above this line
#else
            SetLogger = SetLogger_Native;
            HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
            Unhook = Unhook_Native;
            Tick = Tick_Native;
            ReadRemoteDeviceData = ReadRemoteDeviceData_Native;
            Cleanup = Cleanup_Native;
            // ADD_FUNC: add a statement above this line
#endif
        }

        public void Dispose()
        {
            _l.Dispose();
        }

        public struct RemoteDevice
        {
            public UInt32 id;
            public Vector3 pos;
            public Quaternion quat;
        }
        public Isbl.NetVR.Binary.RemoteDevices ReadRemoteDevices(ulong xrInstance)
        {
            byte[] bytes = null;
            unsafe
            {
                var code = ReadRemoteDeviceData(xrInstance, out var byte_count, out var data);
                if (code != 0) throw new Exception($"ReadRemoteDeviceData failed with error {code}");
                bytes = new byte[byte_count];
                Marshal.Copy(data, bytes, 0, (Int32)byte_count);
                //ReadRemoteDeviceCleanup(byte_count, data);
            }

            return Isbl.NetVR.Binary.RemoteDevices.BincodeDeserialize(bytes);
        }

        public const bool DoesUnload = IsblDynamicLibrary.DoesUnload;
    }

}
