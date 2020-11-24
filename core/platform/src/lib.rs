#![no_std]

pub trait FreeCallback<AsyncResponse> {
    fn call(response: AsyncResponse);
}

pub trait MethodCallback<AsyncResponse> {
    fn call(&self, response: AsyncResponse);
}

pub trait Locator: 'static {
    type T;
    fn locate() -> &'static Self::T;
}

impl<L: Locator, AsyncResponse> FreeCallback<AsyncResponse> for L
where L::T: MethodCallback<AsyncResponse> {
    fn call(response: AsyncResponse) {
        Self::locate().call(response);
    }
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

pub trait Syscalls: 'static {
    unsafe fn raw_const_allow(major: usize, minor: usize, slice: *const u8, len: usize);

    unsafe fn raw_subscribe(major: usize,
                            minor: usize,
                            callback: unsafe extern fn(usize, usize, usize, usize),
                            data: usize);

    fn command(major: usize, minor: usize, arg1: usize, arg2: usize);
    fn yieldk();

    fn const_allow(major: usize, minor: usize, buffer: &'static [u8]) {
        unsafe {
            Self::raw_const_allow(major, minor, buffer.as_ptr(), buffer.len())
        }
    }

    fn subscribe<C: FreeCallback<SubscribeResponse<D>>, D: SubscribeData>(driver: usize,
                                                                          minor: usize,
                                                                          data: D) {
        unsafe {
            Self::raw_subscribe(driver, minor, callback::<C, D>, data.to_usize())
        }
    }
}

unsafe extern fn callback<C: FreeCallback<SubscribeResponse<D>>, D: SubscribeData>(arg1: usize,
                                                                                   arg2: usize,
                                                                                   arg3: usize,
                                                                                   data: usize) {
    C::call(SubscribeResponse { arg1, arg2, arg3, data: D::from_usize(data) });
}
