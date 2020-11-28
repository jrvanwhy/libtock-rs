use crate::HaltBehavior;

/// `SubscribeData` is implemented for types that can be stored by the kernel
/// and passed to `subscribe` callbacks.
///
/// # Safety
///
/// `SubscribeData::from_usize` may be called on a value if and only if that
/// value was produced by `SubscribeData::to_usize` on the same `SubscribeData`
/// implementation (that is, the type that implemented `SubscribeData` must be
/// the same for both conversion directions).
///
/// `from_usize` is `unsafe` because calling `from_usize` on a value that was
/// not produced by `to_usize` (or by `to_usize` on a different type of data)
/// can produce undefined behavior. Further, although `to_usize` is safe to
/// invoke, an incorrect implementation of `to_usize` would lead to undefined
/// behavior, so `SubscribeData` is `unsafe` to implement.
pub unsafe trait SubscribeData {
    fn to_usize(self) -> usize;
    /// # Safety
    /// See above
    unsafe fn from_usize(value: usize) -> Self;
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

/// A default HaltBehavior for `subscribe_once` that panic!()'s with a default
/// failure indication.
pub struct SubscribeOncePanic;

impl HaltBehavior for SubscribeOncePanic {
    fn halt() -> ! {
        panic!("subscribe_once unsubscribe failed");
    }
}

/// When the kernel performs a callback to userspace (registered using
/// `subscribe`), it passes three arguments to the callback as well as a copy of
/// the data that was passed during the `subscribe` call. `Syscalls` converts
/// the passed data into a `SubscribeResponse<D>`, which is passed to the
/// `FreeCallback` that was given to `Subscribe`.
pub struct SubscribeResponse<D: SubscribeData> {
    pub arg1: usize,
    pub arg2: usize,
    pub arg3: usize,
    pub data: D,
}

/// Specifies a particular subscription driver and ID, for use with
/// `subscribe_once`.
pub trait Subscription {
    const DRIVER: usize;
    const ID: usize;
}
