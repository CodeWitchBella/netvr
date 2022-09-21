use crate::xr_struct::XrStruct;

pub struct XrIterator {
    ptr: *const openxr_sys::BaseInStructure,
}

impl XrIterator {
    fn new(ptr: &openxr_sys::BaseInStructure) -> Self {
        Self { ptr }
    }
}

pub trait UnsafeFrom<T> {
    /// .
    ///
    /// # Safety
    ///
    /// ptr must be valid and must not be null. Must point to structure
    /// conforming to OpenXR structure definition (type, next, ...)
    unsafe fn from_ptr(ptr: T) -> Self;
}

macro_rules! implement_from {
    ($t: ty) => {
        impl UnsafeFrom<*const $t> for XrIterator {
            unsafe fn from_ptr(input: *const $t) -> Self {
                XrIterator::new(&*(input as *const openxr_sys::BaseInStructure))
            }
        }
        impl UnsafeFrom<*mut $t> for XrIterator {
            unsafe fn from_ptr(input: *mut $t) -> Self {
                XrIterator::new(&*(input as *const openxr_sys::BaseInStructure))
            }
        }
    };
}

implement_from!(openxr_sys::EventDataBuffer);
implement_from!(openxr_sys::ActionsSyncInfo);
implement_from!(openxr_sys::ActionCreateInfo);

impl Iterator for XrIterator {
    type Item = XrStruct;

    fn next(&mut self) -> Option<Self::Item> {
        let res: *const openxr_sys::BaseInStructure = unsafe { std::mem::transmute(self.ptr) };
        if res.is_null() {
            return None;
        }
        self.ptr = unsafe { std::mem::transmute((*res).next) };
        Some(XrStruct::from(res))
    }
}
