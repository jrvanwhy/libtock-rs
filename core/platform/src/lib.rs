#![no_std]

pub trait Callback<AsyncResponse>: Copy {
    fn locate() -> Self;
    fn call(self, response: AsyncResponse);
}

pub struct SubscribeResponse<D: SubscribeData> {
    pub arg1: usize,
    pub arg2: usize,
    pub arg3: usize,
    pub data: D,
}

pub unsafe trait SubscribeData: Copy {
    fn to_usize(self) -> usize;
    unsafe fn from_usize(value: usize) -> Self;
}

unsafe impl SubscribeData for usize {
    fn to_usize(self) -> usize { self }
    unsafe fn from_usize(value: usize) -> usize { value }
}

unsafe impl<'k, T> SubscribeData for &'k T {
    fn to_usize(self) -> usize { self as *const T as usize }
    unsafe fn from_usize(value: usize) -> &'k T { &*(value as *const T) }
}

pub trait Syscalls<'k>: Copy {
    fn subscribe<C: Callback<SubscribeResponse<D>> + 'k, D: SubscribeData + 'k>(
        self, driver: usize, minor: usize, callback: C, data: D);

    unsafe fn raw_const_allow(self, major: usize, minor: usize, slice: *const u8, len: usize);
    fn command(self, major: usize, minor: usize, arg1: usize, arg2: usize);
    fn yieldk(self);

    fn const_allow(self, major: usize, minor: usize, buffer: &'k [u8]) {
        unsafe {
            self.raw_const_allow(major, minor, buffer.as_ptr(), buffer.len())
        }
    }
}
