use std::fmt;

use crate::{xr_struct::XrStruct, XrDebug, XrDebugValue};

#[derive(Clone)]
pub struct XrIterator {
    ptr: *const openxr_sys::BaseInStructure,
}

impl XrIterator {
    unsafe fn new(ptr: &openxr_sys::BaseInStructure) -> Self {
        Self { ptr }
    }

    /// .
    ///
    /// # Safety
    ///
    /// You must make sure that the clone does not outlive its parent because it
    /// is actually a reference in a trench coat.
    pub(crate) unsafe fn unsafe_clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

pub trait UnsafeFrom<T> {
    /// .
    ///
    /// # Safety
    ///
    /// ptr must be valid and point to structure conforming to OpenXR structure
    /// definition (type, next, ...)
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
implement_from!(openxr_sys::ActionStateGetInfo);

impl Iterator for XrIterator {
    type Item = XrStruct;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let res: *const openxr_sys::BaseInStructure = unsafe { std::mem::transmute(self.ptr) };
        self.ptr = unsafe { std::mem::transmute((*res).next) };
        Some(XrStruct::from(res))
    }
}

#[derive(Clone)]
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
    pub(crate) unsafe fn new(count: u32, ptr: *const T) -> Self {
        Self { count, ptr }
    }
}

impl<T: Copy> Iterator for SizedArrayValueIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        let ptr = self.ptr;
        self.ptr = unsafe { ptr.add(1) };
        self.count -= 1;
        Some(unsafe { *ptr })
    }
}

impl<T> std::fmt::Debug for SizedArrayValueIterator<T>
where
    T: Copy + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut f = f.debug_list();
        for item in self.clone() {
            f.entry(&item);
        }
        f.finish()
    }
}

impl<T> XrDebug for SizedArrayValueIterator<T>
where
    T: Copy + XrDebug,
{
    fn xr_fmt(&self, f: &mut fmt::Formatter, instance: &openxr::Instance) -> fmt::Result {
        let mut f = f.debug_list();
        for item in self.clone() {
            f.entry(&XrDebugValue(instance.clone(), &item));
        }
        f.finish()
    }
}
