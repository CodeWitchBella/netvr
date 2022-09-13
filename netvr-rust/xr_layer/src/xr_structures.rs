use std::{ffi::CStr, os::raw::c_char};

pub struct XrIterator {
    ptr: *const openxr_sys::BaseInStructure,
}

macro_rules! implement_from {
    ($t: ty) => {
        impl From<*const $t> for XrIterator {
            fn from(input: *const $t) -> Self {
                XrIterator {
                    ptr: unsafe { std::mem::transmute(input) },
                }
            }
        }

        impl From<*mut $t> for XrIterator {
            fn from(input: *mut $t) -> Self {
                XrIterator {
                    ptr: unsafe { std::mem::transmute(input) },
                }
            }
        }
    };
}

implement_from!(openxr_sys::EventDataBuffer);
implement_from!(openxr_sys::ActionCreateInfo);

impl Iterator for XrIterator {
    type Item = DecodedStruct;

    fn next(&mut self) -> Option<Self::Item> {
        let res: *const openxr_sys::BaseInStructure = unsafe { std::mem::transmute(self.ptr) };
        if res.is_null() {
            return None;
        }
        self.ptr = unsafe { std::mem::transmute((*res).next) };
        Some(DecodedStruct::from(res))
    }
}

pub struct DecodedStruct {
    pub ty: openxr_sys::StructureType,
    data: *const openxr_sys::BaseInStructure,
}

macro_rules! implement_from {
    ($( $method: ident reads $id: ident), *,) => {
        $(
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct $id<'a>(pub(crate) &'a openxr_sys::$id);
        )*

        impl DecodedStruct {
            $(
                #[allow(dead_code)]
                pub fn $method<'a>(&'a self) -> Option<$id<'a>> {
                    if self.data.is_null() { return None; }
                    Some($id(unsafe {
                        &*std::mem::transmute::<
                            *const openxr_sys::BaseInStructure,
                            *const openxr_sys::$id,
                        >(self.data)
                    }))
                }
            )*
        }

    };
}

implement_from!(
    read_event_data_session_state_changed reads EventDataSessionStateChanged,
    read_event_data_interaction_profile_changed reads EventDataInteractionProfileChanged,
    read_event_data_buffer reads EventDataBuffer,
    read_action_create_info reads ActionCreateInfo,
);

impl DecodedStruct {
    fn from(arg: *const openxr_sys::BaseInStructure) -> Self {
        let ty = unsafe { *arg }.ty;

        Self { ty, data: arg }
    }
}

#[derive(Debug)]
pub enum StringParseError {
    NotNullTerminated,
    Utf8Error(std::str::Utf8Error),
}

fn parse_input_string(name_ptr: &[c_char; 64]) -> Result<&str, StringParseError> {
    if name_ptr[name_ptr.len() - 1] != 0 {
        return Err(StringParseError::NotNullTerminated);
    };
    match unsafe { CStr::from_ptr(name_ptr.as_ptr()) }.to_str() {
        Ok(val) => Ok(val),
        Err(error) => Err(StringParseError::Utf8Error(error)),
    }
}

impl<'a> ActionCreateInfo<'a> {
    pub fn action_name(&'a self) -> Result<&'a str, StringParseError> {
        parse_input_string(&self.0.action_name)
    }

    pub fn action_type(&self) -> openxr::ActionType {
        self.0.action_type
    }
}
