#![no_main]
#![no_std]

use core::cell::Cell;
use core::marker::PhantomPinned;
use core::mem::forget;
use core::ops::{Deref, Range};
use core::pin::{pin, Pin};
use libtock::platform::{ErrorCode, syscall_class, RawSyscalls, Register, return_variant, ReturnVariant, Syscalls};
use libtock::platform::exit_on_drop::ExitOnDrop;
use libtock::runtime::{set_main, stack_size, TockSyscalls};

set_main! {main}
stack_size! {0x100}

mod allow {
    use super::*;

    #[repr(transparent)]
    pub struct RwBuffer<Data: ?Sized, const DRIVER_NUM: u32, const ALLOW_NUM: u32> {
        _pinned: PhantomPinned,
        buffer: Data,
    }
    impl<const DRIVER_NUM: u32, const ALLOW_NUM: u32> AsRef<[u8]> for RwBuffer<[u8], DRIVER_NUM, ALLOW_NUM> {
        fn as_ref(&self) -> &[u8] {
            &self.buffer
        }
    }
    impl<Data: ?Sized, const DRIVER_NUM: u32, const ALLOW_NUM: u32> Drop for RwBuffer<Data, DRIVER_NUM, ALLOW_NUM> {
        fn drop(&mut self) {
            TockSyscalls::unallow_rw(DRIVER_NUM, ALLOW_NUM);
        }
    }
    impl<const LEN: usize, const DRIVER_NUM: u32, const ALLOW_NUM: u32> From<[u8; LEN]> for RwBuffer<[u8; LEN], DRIVER_NUM, ALLOW_NUM> {
        fn from(buffer: [u8; LEN]) -> Self {
            Self {
                _pinned: PhantomPinned,
                buffer,
            }
        }
    }
    impl<const LEN: usize, const DRIVER_NUM: u32, const ALLOW_NUM: u32> RwBuffer<[u8; LEN], DRIVER_NUM, ALLOW_NUM> {
        pub fn get(self: Pin<&mut Self>, range: Range<usize>) -> Option<Pin<&mut RwBuffer<[u8], DRIVER_NUM, ALLOW_NUM>>> {
            let subslice_ptr = unsafe { self.get_unchecked_mut() }.buffer.get_mut(range)? as *mut _ as *mut _;
            Some(unsafe { Pin::new_unchecked(&mut *subslice_ptr) })
        }
        pub fn unallow(self: Pin<&mut Self>) -> [u8; LEN] {
            let this = unsafe { self.get_unchecked_mut() };
            TockSyscalls::unallow_rw(DRIVER_NUM, ALLOW_NUM);
            this.buffer
        }
    }
    impl<const DRIVER_NUM: u32, const ALLOW_NUM: u32> RwBuffer<[u8], DRIVER_NUM, ALLOW_NUM> {
        pub fn allow_rw(self: Pin<&mut Self>) -> Result<(), ErrorCode> {
            let this = unsafe { self.get_unchecked_mut() };
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::ALLOW_RW }>([
                    DRIVER_NUM.into(),
                    ALLOW_NUM.into(),
                    this.buffer.as_mut_ptr().into(),
                    this.buffer.len().into(),
                ])
            };
            let return_variant: ReturnVariant = r0.as_u32().into();
            if return_variant == return_variant::FAILURE_2_U32 {
                // Safety: TRD 104 guarantees that if r0 is Failure with 2 U32,
                // then r1 will contain a valid error code. ErrorCode is
                // designed to be safely transmuted directly from a kernel error
                // code.
                return Err(unsafe { core::mem::transmute(r1.as_u32()) });
            }
            Ok(())
        }
        //pub fn get(self: Pin<&mut Self>, range: Range<usize>) -> Option<Pin<&mut RwBuffer<[u8], DRIVER_NUM, ALLOW_NUM>>> {
        //	let subslice_ptr = unsafe { self.get_unchecked_mut() }.buffer.get_mut(range)? as *mut _ as *mut _;
        //	Some(unsafe { Pin::new_unchecked(&mut *subslice_ptr) })
        //}
    }

    #[repr(transparent)]
    pub struct RoBuffer<Data: ?Sized, const DRIVER_NUM: u32, const ALLOW_NUM: u32> {
        _pinned: PhantomPinned,
        buffer: Data,
    }
    impl<const DRIVER_NUM: u32, const ALLOW_NUM: u32> AsRef<[u8]> for RoBuffer<[u8], DRIVER_NUM, ALLOW_NUM> {
        fn as_ref(&self) -> &[u8] {
            &self.buffer
        }
    }
    impl<Data: ?Sized, const DRIVER_NUM: u32, const ALLOW_NUM: u32> Drop for RoBuffer<Data, DRIVER_NUM, ALLOW_NUM> {
        fn drop(&mut self) {
            TockSyscalls::unallow_ro(DRIVER_NUM, ALLOW_NUM);
        }
    }
    impl<const LEN: usize, const DRIVER_NUM: u32, const ALLOW_NUM: u32> From<[u8; LEN]> for RoBuffer<[u8; LEN], DRIVER_NUM, ALLOW_NUM> {
        fn from(buffer: [u8; LEN]) -> Self {
            Self {
                _pinned: PhantomPinned,
                buffer,
            }
        }
    }
    impl<const LEN: usize, const DRIVER_NUM: u32, const ALLOW_NUM: u32> RoBuffer<[u8; LEN], DRIVER_NUM, ALLOW_NUM> {
        pub fn get(self: Pin<&Self>, range: Range<usize>) -> Option<Pin<&RoBuffer<[u8], DRIVER_NUM, ALLOW_NUM>>> {
            let subslice_ptr = self.buffer.get(range)? as *const _ as *const _;
            Some(unsafe { Pin::new_unchecked(&*subslice_ptr) })
        }
        //pub fn unallow(self: Pin<&Self>) -> [u8; LEN] {
        //	TockSyscalls::unallow_ro(DRIVER_NUM, ALLOW_NUM);
        //	self.get_ref().buffer
        //}
    }
    impl<const DRIVER_NUM: u32, const ALLOW_NUM: u32> RoBuffer<[u8], DRIVER_NUM, ALLOW_NUM> {
        pub fn allow_ro(self: Pin<&Self>) -> Result<(), ErrorCode> {
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::ALLOW_RO }>([
                    DRIVER_NUM.into(),
                    ALLOW_NUM.into(),
                    self.buffer.as_ptr().into(),
                    self.buffer.len().into(),
                ])
            };
            let return_variant: ReturnVariant = r0.as_u32().into();
            if return_variant == return_variant::FAILURE_2_U32 {
                // Safety: TRD 104 guarantees that if r0 is Failure with 2 U32,
                // then r1 will contain a valid error code. ErrorCode is
                // designed to be safely transmuted directly from a kernel error
                // code.
                return Err(unsafe { core::mem::transmute(r1.as_u32()) });
            }
            Ok(())
        }
        //pub fn get(self: Pin<&mut Self>, range: Range<usize>) -> Option<Pin<&mut RoBuffer<[u8], DRIVER_NUM, ALLOW_NUM>>> {
        //	let subslice_ptr = unsafe { self.get_unchecked_mut() }.buffer.get_mut(range)? as *mut _ as *mut _;
        //	Some(unsafe { Pin::new_unchecked(&mut *subslice_ptr) })
        //}
    }
}

mod subscribe {
    use super::*;

    #[derive(Default)]
    pub struct Subscriber<T, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> {
        _pinned: PhantomPinned,
        value: T,
    }
    impl<T, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> AsRef<T> for Subscriber<T, DRIVER_NUM, SUBSCRIBE_NUM> {
        fn as_ref(&self) -> &T {
            &self.value
        }
    }
    impl<T, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> Drop for Subscriber<T, DRIVER_NUM, SUBSCRIBE_NUM> {
        fn drop(&mut self) {
            TockSyscalls::unsubscribe(DRIVER_NUM, SUBSCRIBE_NUM);
        }
    }
    impl<const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> Subscriber<Cell<bool>, DRIVER_NUM, SUBSCRIBE_NUM> {
        pub fn subscribe(self: Pin<&Self>) -> Result<(), ErrorCode> {
            unsafe extern "C" fn kernel_upcall(_: u32, _: u32, _: u32, data: Register) {
                let exit: ExitOnDrop<TockSyscalls> = Default::default();
                let upcall: *const Cell<bool> = data.into();
                unsafe { &*upcall }.set(true);
                forget(exit);
            }
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::SUBSCRIBE }>([
                    DRIVER_NUM.into(),
                    SUBSCRIBE_NUM.into(),
                    (kernel_upcall as *const()).into(),
                    (&self.value as *const Cell::<_>).into(),
                ])
            };
            let return_variant: ReturnVariant = r0.as_u32().into();
            if return_variant == return_variant::FAILURE_2_U32 {
                // Safety: TRD 104 guarantees that if r0 is Failure with 2 U32,
                // then r1 will contain a valid error code. ErrorCode is
                // designed to be safely transmuted directly from a kernel error
                // code.
                return Err(unsafe { core::mem::transmute(r1.as_u32()) });
            }
            Ok(())
        }
    }
    impl<const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> Subscriber<Cell<Option<(u32, u32)>>, DRIVER_NUM, SUBSCRIBE_NUM> {
        pub fn subscribe(self: Pin<&Self>) -> Result<(), ErrorCode> {
            unsafe extern "C" fn kernel_upcall(arg0: u32, arg1: u32, _: u32, data: Register) {
                let exit: ExitOnDrop<TockSyscalls> = Default::default();
                let upcall: *const Cell<Option<(u32, u32)>> = data.into();
                unsafe { &*upcall }.set(Some((arg0, arg1)));
                forget(exit);
            }
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::SUBSCRIBE }>([
                    DRIVER_NUM.into(),
                    SUBSCRIBE_NUM.into(),
                    (kernel_upcall as *const()).into(),
                    (&self.value as *const Cell::<_>).into(),
                ])
            };
            let return_variant: ReturnVariant = r0.as_u32().into();
            if return_variant == return_variant::FAILURE_2_U32 {
                // Safety: TRD 104 guarantees that if r0 is Failure with 2 U32,
                // then r1 will contain a valid error code. ErrorCode is
                // designed to be safely transmuted directly from a kernel error
                // code.
                return Err(unsafe { core::mem::transmute(r1.as_u32()) });
            }
            Ok(())
        }
    }
}

mod alarm {
    use super::*;

    pub fn wakeup(milliseconds: u32, done: Pin<&subscribe::Subscriber<Cell<bool>, 0, 0>>) -> Result<(), ErrorCode> {
        let frequency = TockSyscalls::command(0, 1, 0, 0).to_result::<u32, _>()?;
        let ticks = milliseconds * frequency / 1000;
        done.subscribe()?;
        TockSyscalls::command(0, 5, ticks, 0).to_result::<u32, _>()?;
        Ok(())
    }
}

mod console {
    use super::*;

    pub fn read(buffer: Pin<&mut allow::RwBuffer<[u8], 1, 1>>, bytes: Pin<&subscribe::Subscriber<Cell<Option<(u32, u32)>>, 1, 2>>) -> Result<(), ErrorCode> {
        bytes.subscribe()?;
        let len = buffer.deref().as_ref().len() as u32;
        buffer.allow_rw()?;
        TockSyscalls::command(1, 2, len, 0).to_result()
    }

    pub fn write(buffer: Pin<&allow::RoBuffer<[u8], 1, 1>>, done: Pin<&subscribe::Subscriber<Cell<bool>, 1, 1>>) -> Result<(), ErrorCode> {
        buffer.allow_ro()?;
        done.subscribe()?;
        TockSyscalls::command(1, 1, buffer.deref().as_ref().len() as u32, 0).to_result()
    }
}

fn main() -> Result<(), ErrorCode> {
    let buffer: [u8; 25] = *b"Hello,                   ";
    let buffer: allow::RwBuffer<_, 1, 1> = buffer.into();
    let mut buffer = pin!(buffer);
    let bytes: Pin<&mut subscribe::Subscriber<_, 1, 2>> = pin!(Default::default());
    let timeout: Pin<&mut subscribe::Subscriber<_, 0, 0>> = pin!(Default::default());
    console::read(buffer.as_mut().get(7..23).unwrap(), bytes.as_ref())?;
    alarm::wakeup(1000, timeout.as_ref())?;
    let receive_len = loop {
        TockSyscalls::yield_wait();
        if let Some((_, len)) = bytes.deref().as_ref().get() {
            break len as usize;
        }
        if timeout.deref().as_ref().get() {
            break 0;
        }
    };
    // Hmm, these two drop calls seem easy to forget.
    drop(bytes);
    drop(timeout);
    let mut buffer = buffer.unallow();
    // Console *shouldn't* return a receive_len greater than 16, but if it bugs
    // out it could. This check is needed to prevent subsequent indexing
    // operations from panicing, which allows the compiler to optimize the
    // panics away.
    if receive_len > 16 {
        return Err(ErrorCode::Fail);
    }
    let name_len = match receive_len {
        0 => {
            let world: &mut [u8; 5] = (&mut buffer[7..12]).try_into().unwrap();
            *world = *b"World";
            world.len()
        },
        len => len,
    };
    buffer[7 + name_len] = b'!';
    buffer[8 + name_len] = b'\n';
    let msg_len = 9 + name_len;
    let done: Pin<&mut subscribe::Subscriber<_, 1, 1>> = pin!(Default::default());
    let buffer: allow::RoBuffer<_, 1, 1> = buffer.into();
    let buffer = pin!(buffer);
    let to_print = buffer.as_ref().get(0..msg_len).unwrap();
    console::write(to_print, done.as_ref())?;
    loop {
        TockSyscalls::yield_wait();
    }
}
