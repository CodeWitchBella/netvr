use crate::xr_wrap::{xr_wrap, ResultConvertible, XrWrapError};
use netvr_data::{bincode, serde};
use std::alloc;
use xr_layer::sys;

/// Utility function to handle the bincode ABI. Deserializes input, calls the
/// function, serializes the output and handles errors and panics.
pub(crate) fn bincode_abi<'de, Input, Output, O>(
    length: *mut u32,
    data: *mut *mut u8,
    function: O,
) -> sys::Result
where
    O: FnOnce(Input) -> Result<Output, XrWrapError>,
    O: std::panic::UnwindSafe,
    Input: serde::Deserialize<'de>,
    Output: serde::Serialize,
{
    xr_wrap(|| {
        let slice = unsafe { std::slice::from_raw_parts(*data, { *length }.try_into()?) };
        let input: Input = bincode::deserialize(slice)?;

        let output = function(input)?;
        let encoded: Vec<u8> = bincode::serialize(&output)?;
        let u32len: u32 = encoded.len().try_into()?;

        let layout = alloc::Layout::from_size_align(encoded.len(), 1)?;
        unsafe {
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                return sys::Result::ERROR_OUT_OF_MEMORY.into_result();
            }
            std::ptr::copy(encoded.as_ptr(), ptr, encoded.len());
            length.write(u32len);
            data.write(ptr);
        };
        Ok(())
    })
}

/// Part of the bincode ABI. This is called from the caller to free the memory
/// used to pass data back to the caller.
#[no_mangle]
pub extern "C" fn netvr_cleanup(length: u32, data: *mut u8) {
    let layout = alloc::Layout::from_size_align(length.try_into().unwrap(), 1).unwrap();
    unsafe {
        alloc::dealloc(data.try_into().unwrap(), layout);
    }
}

/// Generates the following:
///  - netvr_get_fn
///  - codegen
macro_rules! bincode_expose {
    ($(expose $id: ident as $cap: ident taking $input: ident and outputting $output: ident), *,) => {
        mod abi {
            use super::bincode_abi::bincode_abi;
            use xr_layer::sys;

            $(
                #[allow(dead_code)]
                pub(super) unsafe extern "C" fn $id(
                    length: *mut u32,
                    data: *mut *mut u8,
                ) -> sys::Result {
                    bincode_abi::<netvr_data::$input, netvr_data::$output, _>(length, data, super::$id)
                }
            )*
        }

        #[cfg(crate_type = "lib")]
        #[no_mangle]
        pub unsafe extern "C" fn netvr_get_fn(
            name_cstr: *const std::ffi::c_char,
            function: *mut Option<unsafe extern "C" fn(*mut u32, *mut *mut u8) -> sys::Result>,
        ) {
            LogInfo::cstr(name_cstr);
            function.write(None);
            if let Ok(name) = unsafe { std::ffi::CStr::from_ptr(name_cstr) }.to_str() {
                LogInfo::string(format!("Getting {name:?}"));
                match name {
                    $(
                        stringify!($id) => {
                            function.write(Some(abi::$id));
                        }
                    )*
                    _ => {}
                };
            }
        }

        #[cfg(not(crate_type = "lib"))]
        pub fn codegen() -> String {
            let mut code = r#"using System;
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

        "#.to_owned();
            $(
                code.push_str("\n        private BincodeABI_Delegate Cache_");
                code.push_str(stringify!($cap));
                code.push_str(";\n");
                code.push_str("        public ");
                code.push_str(stringify!($output));
                code.push_str(" ");
                code.push_str(stringify!($cap));
                code.push_str("(");
                code.push_str(stringify!($input));
                code.push_str(" input) => ");
                code.push_str(stringify!($output));
                code.push_str(".BincodeDeserialize(FunctionCall(input.BincodeSerialize(), ref Cache_");
                code.push_str(stringify!($cap));
                code.push_str(", \"");
                code.push_str(stringify!($id));
                code.push_str("\"));\n");
            )*
            code.push_str("    }\n}\n");
            code
        }
    };
}
