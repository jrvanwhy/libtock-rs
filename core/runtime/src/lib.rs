#![feature(llvm_asm)]
#![no_std]

#[derive(Clone, Copy)]
pub struct TockSyscalls;

impl libtock_platform::Syscalls<'static> for TockSyscalls {
    fn raw_subscribe(self, driver: usize, minor: usize, callback: extern "C" fn(usize, usize, usize, usize), data: usize) {
        let res: usize;
        unsafe {
            llvm_asm!(
                "li a0, 1
                ecall"
                : "={x10}"(res)
                : "{x11}"(driver), "{x12}"(minor), "{x13}"(callback), "{x14}"(data)
                : "memory"
                : "volatile");
        }
        let _ = res;
    }

    unsafe fn raw_const_allow(self, major: usize, minor: usize, slice: *const u8, len: usize) {
        let res: usize;
        llvm_asm!("li    a0, 3
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (slice), "{x14}" (len)
         : "memory"
         : "volatile");
        let _ = res;
    }

    fn command(self, major: usize, minor: usize, arg1: usize, arg2: usize) {
        let res: usize;
        unsafe {
            llvm_asm!("li    a0, 2
                  ecall"
                 : "={x10}" (res)
                 : "{x11}" (major), "{x12}" (minor), "{x13}" (arg1), "{x14}" (arg2)
                 : "memory"
                 : "volatile");
        }
        let _ = res;
    }

    fn yieldk(self) {
        let res: usize;
        unsafe {
            llvm_asm! (
                    "li    a0, 0
                    ecall"
                    :
                    :
                    : "memory", "x10", "x11", "x12", "x13", "x14", "x15", "x16", "x17",
                    "x5", "x6", "x7", "x28", "x29", "x30", "x31", "x1"
                    : "volatile");
        }
        let _ = res;
    }
}

#[macro_export]
macro_rules! static_component {
    [$link:ident, $name:ident: $comp:ty = $init:expr] => {
        static mut COMPONENT: $comp = $init;
        struct $link;
    };
}
