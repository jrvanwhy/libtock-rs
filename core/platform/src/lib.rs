#![no_std]

pub trait Syscalls<'k>: Copy {
    fn raw_subscribe(self, driver: usize, minor: usize, callback: extern fn (usize, usize, usize, usize), data: usize);
    unsafe fn raw_const_allow(self, major: usize, minor: usize, slice: *const u8, len: usize);
    fn command(self, major: usize, minor: usize, arg1: usize, arg2: usize);
    fn yieldk(self);

    fn subscribe<Callback: SubscribeCallback + 'k>(self,
                                                   driver: usize,
                                                   minor: usize,
                                                   data: usize) {
        self.raw_subscribe(driver, minor, callback::<Callback>, data);
    }

    fn const_allow(self, major: usize, minor: usize, slice: &'k [u8]) {
        unsafe {
            self.raw_const_allow(major, minor, slice.as_ptr(), slice.len())
        }
    }
}

pub struct CallbackContext<'c> { _private: core::marker::PhantomData<&'c ()> }

pub trait SubscribeCallback {
    extern fn callback(context: CallbackContext, arg1: usize, arg2: usize, arg3: usize, data: usize);
}

extern fn callback<Callback: SubscribeCallback>(arg1: usize, arg2: usize, arg3: usize, data: usize) {
    Callback::callback(CallbackContext { _private: core::marker::PhantomData }, arg1, arg2, arg3, data);
}
