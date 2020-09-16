#![no_std]

mod callbacks;

pub use callbacks::{FreeCallback, MethodCallback};

pub trait Syscalls: Copy {
    fn subscribe(self, driver: usize, minor: usize, callback: extern "C" fn(usize, usize, usize, usize), data: usize);
    unsafe fn const_allow(self, major: usize, minor: usize, slice: *const u8, len: usize);
    fn command(self, major: usize, miner: usize, arg1: usize, arg2: usize);
    fn yieldk(self);
}
