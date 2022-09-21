use std::{ffi::CStr, fmt::Debug, os::raw::c_char};

pub struct XrStruct {
    pub ty: openxr_sys::StructureType,
    data: *const openxr_sys::BaseInStructure,
}

macro_rules! implement_readers {
    ($( $method: ident reads $id: ident), *,) => {
        $(
            #[repr(transparent)]
            pub struct $id<'a>(&'a openxr_sys::$id);

            impl<'a> $id<'a> {
                #[inline]
                pub fn as_raw(&'a self) -> &'a openxr_sys::$id {
                    self.0
                }
            }

            impl<'a> std::fmt::Debug for $id<'a> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }
        )*

        impl XrStruct {
            $(
                #[allow(dead_code)]
                pub fn $method<'a>(&'a self) -> Option<$id<'a>> {
                    if self.data.is_null() { return None; }
                    if self.ty != openxr_sys::$id::TYPE { return None; }
                    Some($id(unsafe {
                        &*std::mem::transmute::<
                            *const openxr_sys::BaseInStructure,
                            *const openxr_sys::$id,
                        >(self.data)
                    }))
                }
            )*
        }

        impl<'a> crate::XrDebug<'a> for XrStruct {
            fn xr_debug(&'a self, instance: &openxr::Instance) -> crate::XrDebugValue<'a> {
                crate::XrDebugValue::new(instance.clone(), |debugable, f| match self.ty {
                    $(
                        openxr_sys::$id::TYPE => {
                            match self.$method() {
                                Some(v) => v.xr_debug(&debugable.instance).fmt(f),
                                None => f.debug_struct("None").finish(),
                            }
                        }
                    )*
                    _ => f
                        .debug_struct("Unknown")
                        .field("ty", &self.ty)
                        .finish_non_exhaustive(),
                })
            }
        }
    };
}

implement_readers!(
    read_event_data_session_state_changed reads EventDataSessionStateChanged,
    read_event_data_interaction_profile_changed reads EventDataInteractionProfileChanged,
    read_event_data_buffer reads EventDataBuffer,
    read_action_create_info reads ActionCreateInfo,
);

impl XrStruct {
    pub(crate) fn from(arg: *const openxr_sys::BaseInStructure) -> Self {
        let ty = unsafe { *arg }.ty;

        Self { ty, data: arg }
    }
}

#[derive(Debug)]
pub enum StringParseError {
    NotNullTerminated,
    Utf8Error(std::str::Utf8Error),
}

fn parse_input_string(name_ptr: &[c_char]) -> Result<&str, StringParseError> {
    if name_ptr[name_ptr.len() - 1] != 0 {
        return Err(StringParseError::NotNullTerminated);
    };
    match unsafe { CStr::from_ptr(name_ptr.as_ptr()) }.to_str() {
        Ok(val) => Ok(val),
        Err(error) => Err(StringParseError::Utf8Error(error)),
    }
}

pub struct SizedArrayValueIterator<T>
where
    T: Copy,
{
    count: u32,
    ptr: *const T,
}

impl<T> SizedArrayValueIterator<T>
where
    T: Copy,
{
    unsafe fn new(count: u32, ptr: *const T) -> Self {
        Self { count, ptr }
    }
}

impl<T> Iterator for SizedArrayValueIterator<T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        let ptr = self.ptr;
        self.ptr = unsafe { ptr.add(1) };
        self.count = 0;
        Some(unsafe { *ptr })
    }
}

impl<'a> ActionCreateInfo<'a> {
    pub fn action_name(&'a self) -> Result<&'a str, StringParseError> {
        parse_input_string(&self.0.action_name)
    }

    pub fn localized_action_name(&'a self) -> Result<&'a str, StringParseError> {
        parse_input_string(&self.0.localized_action_name)
    }

    pub fn action_type(&self) -> openxr_sys::ActionType {
        self.0.action_type
    }

    pub fn subaction_paths(&self) -> SizedArrayValueIterator<openxr_sys::Path> {
        unsafe {
            SizedArrayValueIterator::new(self.0.count_subaction_paths, self.0.subaction_paths)
        }
    }
}
