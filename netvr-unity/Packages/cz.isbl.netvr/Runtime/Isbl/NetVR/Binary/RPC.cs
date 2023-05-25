using System;
using System.Runtime.InteropServices;

namespace Isbl.NetVR.Binary 
{
    class RPC 
    {
        public unsafe delegate System.Int32 BincodeABI_Delegate(ref System.UInt32 length, ref byte* data);
        public delegate void GetFn_Delegate([MarshalAs(UnmanagedType.LPStr)] string name, out BincodeABI_Delegate fn);
        public unsafe delegate void Cleanup_Delegate(System.UInt32 length, byte* data);
        private readonly GetFn_Delegate GetFn;
        private readonly Cleanup_Delegate Cleanup;
        public RPC(GetFn_Delegate getFn, Cleanup_Delegate cleanup)
        {
            GetFn = getFn;
            Cleanup = cleanup;
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
                    if (code != 0) throw new Exception($"Function {name} failed with error {code}");
                    if (data == null || data == bytes_ptr) return null;
                    bytes = new byte[byte_count];
                    Marshal.Copy((IntPtr)data, bytes, 0, (Int32)byte_count);
                    Cleanup(byte_count, data);
                };
            }
            return bytes;
        }

        
        private BincodeABI_Delegate Cache_ReadRemoteDevices;
        public ReadRemoteDevicesOutput ReadRemoteDevices(InstanceAndSession input) => ReadRemoteDevicesOutput.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_ReadRemoteDevices, "read_remote_devices"));

        private BincodeABI_Delegate Cache_Start;
        public Nothing Start(StartInput input) => Nothing.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_Start, "start"));

        private BincodeABI_Delegate Cache_ReadRemoteObjects;
        public Snapshot ReadRemoteObjects(InstanceAndSession input) => Snapshot.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_ReadRemoteObjects, "read_remote_objects"));

        private BincodeABI_Delegate Cache_InitRemoteObjects;
        public Nothing InitRemoteObjects(InitRemoteObjectsInput input) => Nothing.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_InitRemoteObjects, "init_remote_objects"));

        private BincodeABI_Delegate Cache_Grab;
        public Nothing Grab(GrabInput input) => Nothing.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_Grab, "grab"));

        private BincodeABI_Delegate Cache_Release;
        public Nothing Release(GrabInput input) => Nothing.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_Release, "release"));

        private BincodeABI_Delegate Cache_ObjectSetPose;
        public Nothing ObjectSetPose(SetPoseInput input) => Nothing.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_ObjectSetPose, "object_set_pose"));

        private BincodeABI_Delegate Cache_GetServerAddress;
        public OnlyString GetServerAddress(InstanceAndSession input) => OnlyString.BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_GetServerAddress, "get_server_address"));
    }
}
