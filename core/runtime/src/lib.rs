#![feature(llvm_asm)]
#![no_std]

pub struct TockSyscalls;

impl libtock_platform::Syscalls for TockSyscalls {
    unsafe fn raw_const_allow(major: usize, minor: usize, slice: *const u8, len: usize) {
        let res: usize;
        llvm_asm!("li    a0, 3
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (slice), "{x14}" (len)
         : "memory"
         : "volatile");
        let _ = res;
    }

    unsafe fn raw_subscribe(major: usize, minor: usize, callback: unsafe extern fn(usize, usize, usize, usize), data: usize) {
        let res: usize;
        llvm_asm!(
            "li a0, 1
            ecall"
            : "={x10}"(res)
            : "{x11}"(major), "{x12}"(minor), "{x13}"(callback), "{x14}"(data)
            : "memory"
            : "volatile");
        let _ = res;
    }

    fn command(major: usize, minor: usize, arg1: usize, arg2: usize) {
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

    fn yieldk() {
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

// Should this be called SyncWrap?
#[repr(transparent)]
pub struct TockStatic<T> {
    value: T,
}

impl<T> TockStatic<T> {
    pub const fn new(value: T) -> TockStatic<T> {
        TockStatic { value }
    }
}

unsafe impl<T> Sync for TockStatic<T> {}

impl<T> core::ops::Deref for TockStatic<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

#[macro_export]
macro_rules! static_component {
    [$locator:ident; $name:ident: $comp:ty = $init:expr] => {
        static $name: $crate::TockStatic<$comp> = $crate::TockStatic::new($init);
        struct $locator;
        impl libtock_platform::Locator for $locator {
            type T = $comp;
            fn locate() -> &'static $comp { &$name }
        }
    };
}
