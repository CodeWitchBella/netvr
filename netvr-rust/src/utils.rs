use std::os::raw::c_char;
pub type Cstr = *const c_char;

#[macro_export]
macro_rules! internal_screaming {
    () => {
        panic!("AAAAAAAAAAAaaaaaaaaaaaaaaaaaaaaaaaaa");
    };
}

pub fn xr_wrap<O>(function: O) -> openxr_sys::Result
where
    O: FnOnce() -> Result<(), openxr_sys::Result>,
{
    match function() {
        Ok(()) => openxr_sys::Result::SUCCESS,
        Err(v) => v,
    }
}

pub(crate) trait ResultConvertible {
    fn into_result(self) -> Result<(), openxr_sys::Result>;
}

impl ResultConvertible for openxr_sys::Result {
    fn into_result(self) -> Result<(), openxr_sys::Result> {
        if self == openxr_sys::Result::SUCCESS {
            Ok(())
        } else {
            Err(self)
        }
    }
}
