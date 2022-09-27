use std::{os::raw::c_char, panic};

use crate::{log::LogWarn, XrResult};
pub type Cstr = *const c_char;

#[macro_export]
macro_rules! internal_screaming {
    () => {
        panic!("AAAAAAAAAAAaaaaaaaaaaaaaaaaaaaaaaaaa");
    };
}

pub(crate) trait ResultConvertible {
    fn into_result(self) -> XrResult<()>;
}

impl ResultConvertible for openxr_sys::Result {
    fn into_result(self) -> XrResult<()> {
        if self == openxr_sys::Result::SUCCESS {
            Ok(())
        } else {
            Err(self)
        }
    }
}

pub(crate) trait ResultToWarning {
    fn warn_on_err(self, function_name: &'static str);
}

impl ResultToWarning for XrResult<()> {
    fn warn_on_err(self, function_name: &'static str) {
        if let Err(error) = self {
            LogWarn::string(format!(
                "Function {} failed with result {:?}",
                function_name, error
            ))
        }
    }
}
