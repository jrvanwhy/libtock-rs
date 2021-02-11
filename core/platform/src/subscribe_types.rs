/// The `subscribe` system calls allow processes to give a register-sized value
/// that is passed to the callbacks when they are invoked. `SubscribeData` is
/// implemented for types that can be soundly passed to the callback through
/// `subscribe`.
// to_usize is a safe function, but if it is implemented incorrectly it can
// cause undefined behavior. Therefore, SubscribeData is an unsafe trait to
// implement. Its implementation must satisfy the following invariant:
//
// If value is of type T, then
// <T as SubscribeData>::from_usize(value.to_usize())
// is does not produce undefined behavior and returns the original value.
//
// There are a couple gotchas that you need to look out for when designing a
// safe interface to `subscribe`:
//     1. SubscribeData does not require the type to be Copy. If your interface
//        can result in multiple callbacks, then you need to ensure the data is
//        Copy.
//     2. SubscribeData does not bound the lifetime of the data.
pub unsafe trait SubscribeData {
    fn to_usize(self) -> usize;
    unsafe fn from_usize(value: usize) -> Self;
}

unsafe impl SubscribeData for u32 {
    fn to_usize(self) -> usize {
        self as usize
    }
    unsafe fn from_usize(value: usize) -> u32 {
        value as u32
    }
}

unsafe impl SubscribeData for usize {
    fn to_usize(self) -> usize {
        self
    }
    unsafe fn from_usize(value: usize) -> usize {
        value
    }
}

unsafe impl<'k, T> SubscribeData for &'k T {
    fn to_usize(self) -> usize {
        self as *const T as usize
    }
    unsafe fn from_usize(value: usize) -> &'k T {
        &*(value as *const T)
    }
}

unsafe impl<'k, T> SubscribeData for &'k mut T {
    fn to_usize(self) -> usize {
        self as *mut T as usize
    }
    unsafe fn from_usize(value: usize) -> &'k mut T {
        &mut *(value as *mut T)
    }
}

/// `SubscribeResponse` contains all the data the kernel passes to a callback.
pub struct SubscribeResponse<D: SubscribeData> {
    pub arg1: u32,
    pub arg2: u32,
    pub arg3: u32,
    pub data: D,
}
