use std::fmt;

use crate::{XrDebug, XrDebugValue};

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
    /// # Safety
    /// `ptr` must be a valid pointer to `count` elements of type `T`.
    pub unsafe fn new(count: u32, ptr: *const T) -> Self {
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
