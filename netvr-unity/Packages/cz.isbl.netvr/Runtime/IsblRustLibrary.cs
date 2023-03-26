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

        private unsafe delegate System.Int32 ReadRemoteDeviceData_Delegate(ulong xrInstance, System.UInt32 length, byte* data);
        private readonly ReadRemoteDeviceData_Delegate ReadRemoteDeviceData;

        private delegate System.UInt32 ReadRemoteDeviceCount_Delegate(ulong xrInstance);
        private readonly ReadRemoteDeviceCount_Delegate ReadRemoteDeviceCount;
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
        static unsafe extern System.Int32 ReadRemoteDeviceData_Native(ulong xrInstance, System.UInt32 length, byte* data);

        [DllImport(LibraryName, EntryPoint = "netvr_read_remote_device_count")]
        static extern System.UInt32 ReadRemoteDeviceCount_Native(ulong xrInstance);
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
            _l.GetDelegate("netvr_read_remote_device_count", out ReadRemoteDeviceCount);
            // ADD_FUNC: add GetDelegate call above this line
#else
            SetLogger = SetLogger_Native;
            HookGetInstanceProcAddr = HookGetInstanceProcAddr_Native;
            Unhook = Unhook_Native;
            Tick = Tick_Native;
            ReadRemoteDeviceData = ReadRemoteDeviceData_Native;
            ReadRemoteDeviceCount = ReadRemoteDeviceCount_Native;
            // ADD_FUNC: add a statement above this line
#endif
        }

        public void Dispose()
        {
            _l.Dispose();
        }

        public struct RemoteDevice {
            public UInt32 id;
            public Vector3 pos;
            public Quaternion quat;
        }
        public RemoteDevice[] ReadRemoteDevices(ulong xrInstance) {
            var count = ReadRemoteDeviceCount(xrInstance);
            if (count < 1) return new RemoteDevice[0];
            const int device_bytes = 4 * 8;
            var bytes = new byte[count * device_bytes];
            MemoryStream stream = new(bytes);
            BinaryReader reader = new(stream);
            Int32 code;
            unsafe {
                fixed (byte* p = bytes)
                {
                    code = ReadRemoteDeviceData(xrInstance, (UInt32)bytes.Length, p);
                }
            }
            if (code != 0) throw new Exception($"ReadRemoteDeviceData failed with error {code}");
            var devices = new RemoteDevice[count];
            for (var i = 0; i < count; ++i) {
                devices[i].id = reader.ReadUInt32();
                devices[i].pos = new Vector3(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
                devices[i].quat = new Quaternion(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
                if(stream.Position != device_bytes*(i+1)) throw new Exception($"Incorrect Position. Expected: {device_bytes*i} got: {stream.Position}");
            }
            return devices;
        }

        public const bool DoesUnload = IsblDynamicLibrary.DoesUnload;
    }

}
