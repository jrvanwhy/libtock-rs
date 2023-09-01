#![no_main]
#![no_std]

use core::cell::Cell;
use core::marker::{PhantomData, PhantomPinned};
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

    pub trait RoId {
        const DRIVER_NUM: u32;
        const ALLOW_NUM: u32;
    }

    pub struct RoBuffer<ID: RoId, const LEN: usize> {
        buffer: [u8; LEN],
        needs_unallow: bool,
        _phantom: (PhantomData<ID>, PhantomPinned),
    }
    impl<ID: RoId, const LEN: usize> Drop for RoBuffer<ID, LEN> {
        fn drop(&mut self) {
            // Could implement by calling self.unallow or a function shared
            // between the two -- maybe smaller if not inlined?
            if self.needs_unallow {
                TockSyscalls::unallow_ro(ID::DRIVER_NUM, ID::ALLOW_NUM);
            }
        }
    }
    impl<ID: RoId, const LEN: usize> RoBuffer<ID, LEN> {
        pub fn new(buffer: [u8; LEN]) -> Self {
            RoBuffer {
                buffer,
                needs_unallow: false,
                _phantom: Default::default(),
            }
        }

        #[allow(unused)]
        pub fn buffer(&mut self) -> &mut [u8; LEN] {
            &mut self.buffer
        }

        pub fn slice(self: Pin<&mut Self>) -> &mut RoSlice<ID> {
            let this = unsafe { self.get_unchecked_mut() };
            this.needs_unallow = true;
            unsafe { &mut *(&mut this.buffer as *mut [u8] as *mut _) }
        }

        #[allow(unused)]
        pub fn unallow(self: Pin<&mut Self>) -> &mut Self {
            let this = unsafe { self.get_unchecked_mut() };
            // Doesn't need to be optional...
            if this.needs_unallow {
                TockSyscalls::unallow_ro(ID::DRIVER_NUM, ID::ALLOW_NUM);
            }
            this.needs_unallow = false;
            this
        }
    }

    #[repr(transparent)]
    pub struct RoSlice<ID: RoId> {
        _phantom: PhantomData<ID>,
        slice: [u8],
    }
    impl<ID: RoId> AsRef<[u8]> for RoSlice<ID> {
        fn as_ref(&self) -> &[u8] {
            &self.slice
        }
    }
    impl<ID: RoId> RoSlice<ID> {
        pub fn allow_ro(&mut self) -> Result<(), ErrorCode> {
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::ALLOW_RO }>([
                    ID::DRIVER_NUM.into(),
                    ID::ALLOW_NUM.into(),
                    self.slice.as_mut_ptr().into(),
                    self.slice.len().into(),
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

        pub fn get(&mut self, range: Range<usize>) -> Option<&mut Self> {
            let pointer = self.slice.get_mut(range)? as *mut [u8] as *mut Self;
            Some(unsafe { &mut *pointer })
        }
    }

    pub trait RwId {
        const DRIVER_NUM: u32;
        const ALLOW_NUM: u32;
    }

    pub struct RwBuffer<ID: RwId, const LEN: usize> {
        buffer: [u8; LEN],
        needs_unallow: bool,
        _phantom: (PhantomData<ID>, PhantomPinned),
    }
    impl<ID: RwId, const LEN: usize> Drop for RwBuffer<ID, LEN> {
        fn drop(&mut self) {
            // Could implement by calling self.unallow or a function shared
            // between the two -- maybe smaller if not inlined?
            if self.needs_unallow {
                TockSyscalls::unallow_rw(ID::DRIVER_NUM, ID::ALLOW_NUM);
            }
        }
    }
    impl<ID: RwId, const LEN: usize> RwBuffer<ID, LEN> {
        pub fn new(buffer: [u8; LEN]) -> Self {
            RwBuffer {
                buffer,
                needs_unallow: false,
                _phantom: Default::default(),
            }
        }

        pub fn buffer(&mut self) -> &mut [u8; LEN] {
            &mut self.buffer
        }

        pub fn slice(self: Pin<&mut Self>) -> &mut RwSlice<ID> {
            let this = unsafe { self.get_unchecked_mut() };
            this.needs_unallow = true;
            unsafe { &mut *(&mut this.buffer as *mut [u8] as *mut _) }
        }

        pub fn unallow(self: Pin<&mut Self>) -> &mut Self {
            let this = unsafe { self.get_unchecked_mut() };
            // Doesn't need to be optional...
            if this.needs_unallow {
                TockSyscalls::unallow_rw(ID::DRIVER_NUM, ID::ALLOW_NUM);
            }
            this.needs_unallow = false;
            this
        }
    }

    #[repr(transparent)]
    pub struct RwSlice<ID: RwId> {
        _phantom: PhantomData<ID>,
        slice: [u8],
    }
    impl<ID: RwId> AsRef<[u8]> for RwSlice<ID> {
        fn as_ref(&self) -> &[u8] {
            &self.slice
        }
    }
    impl<ID: RwId> RwSlice<ID> {
        pub fn allow_rw(&mut self) -> Result<(), ErrorCode> {
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::ALLOW_RW }>([
                    ID::DRIVER_NUM.into(),
                    ID::ALLOW_NUM.into(),
                    self.slice.as_mut_ptr().into(),
                    self.slice.len().into(),
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

        pub fn get(&mut self, range: Range<usize>) -> Option<&mut Self> {
            let pointer = self.slice.get_mut(range)? as *mut [u8] as *mut Self;
            Some(unsafe { &mut *pointer })
        }
    }
}

mod subscribe {
    use super::*;

    pub trait Id {
        const DRIVER_NUM: u32;
        const SUBSCRIBE_NUM: u32;
    }

    pub struct Subscriber<ID: Id, T> {
        value: T,
        _phantom: (PhantomData<ID>, PhantomPinned),
    }
    impl<ID: Id, T: Default> Default for Subscriber<ID, T> {
        fn default() -> Self {
            Subscriber {
                value: Default::default(),
                _phantom: Default::default(),
            }
        }
    }
    impl<ID: Id, T> AsRef<T> for Subscriber<ID, T> {
        fn as_ref(&self) -> &T {
            &self.value
        }
    }
    impl<ID: Id, T> Drop for Subscriber<ID, T> {
        fn drop(&mut self) {
            TockSyscalls::unsubscribe(ID::DRIVER_NUM, ID::SUBSCRIBE_NUM);
        }
    }
    impl<ID: Id> Subscriber<ID, Cell<bool>> {
        pub fn subscribe(self: Pin<&Self>) -> Result<(), ErrorCode> {
            unsafe extern "C" fn kernel_upcall(_: u32, _: u32, _: u32, data: Register) {
                let exit: ExitOnDrop<TockSyscalls> = Default::default();
                let upcall: *const Cell<bool> = data.into();
                unsafe { &*upcall }.set(true);
                forget(exit);
            }
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::SUBSCRIBE }>([
                    ID::DRIVER_NUM.into(),
                    ID::SUBSCRIBE_NUM.into(),
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
    impl<ID: Id> Subscriber<ID, Cell<Option<(u32, u32)>>> {
        pub fn subscribe(self: Pin<&Self>) -> Result<(), ErrorCode> {
            unsafe extern "C" fn kernel_upcall(arg0: u32, arg1: u32, _: u32, data: Register) {
                let exit: ExitOnDrop<TockSyscalls> = Default::default();
                let upcall: *const Cell<Option<(u32, u32)>> = data.into();
                unsafe { &*upcall }.set(Some((arg0, arg1)));
                forget(exit);
            }
            let [r0, r1, _, _] = unsafe {
                TockSyscalls::syscall4::<{ syscall_class::SUBSCRIBE }>([
                    ID::DRIVER_NUM.into(),
                    ID::SUBSCRIBE_NUM.into(),
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

    pub struct WakeupId;
    impl subscribe::Id for WakeupId {
        const DRIVER_NUM: u32 = 0;
        const SUBSCRIBE_NUM: u32 = 0;
    }
    pub fn wakeup(milliseconds: u32, done: Pin<&subscribe::Subscriber<WakeupId, Cell<bool>>>) -> Result<(), ErrorCode> {
        let frequency = TockSyscalls::command(0, 1, 0, 0).to_result::<u32, _>()?;
        let ticks = milliseconds * frequency / 1000;
        done.subscribe()?;
        TockSyscalls::command(0, 5, ticks, 0).to_result::<u32, _>()?;
        Ok(())
    }
}

mod console {
    use super::*;

    pub struct ReadId;
    impl allow::RwId for ReadId {
        const DRIVER_NUM: u32 = 1;
        const ALLOW_NUM: u32 = 1;
    }
    impl subscribe::Id for ReadId {
        const DRIVER_NUM: u32 = 1;
        const SUBSCRIBE_NUM: u32 = 2;
    }
    pub fn read(buffer: &mut allow::RwSlice<ReadId>, bytes: Pin<&subscribe::Subscriber<ReadId, Cell<Option<(u32, u32)>>>>) -> Result<(), ErrorCode> {
        bytes.subscribe()?;
        let len = buffer.deref().as_ref().len() as u32;
        buffer.allow_rw()?;
        TockSyscalls::command(1, 2, len, 0).to_result()
    }

    pub struct WriteId;
    impl allow::RoId for WriteId {
        const DRIVER_NUM: u32 = 1;
        const ALLOW_NUM: u32 = 1;
    }
    impl subscribe::Id for WriteId {
        const DRIVER_NUM: u32 = 1;
        const SUBSCRIBE_NUM: u32 = 1;
    }
    pub fn write(buffer: &mut allow::RoSlice<WriteId>, done: Pin<&subscribe::Subscriber<WriteId, Cell<bool>>>) -> Result<(), ErrorCode> {
        buffer.allow_ro()?;
        done.subscribe()?;
        TockSyscalls::command(1, 1, buffer.deref().as_ref().len() as u32, 0).to_result()
    }
}

fn main() -> Result<(), ErrorCode> {
    let buffer = allow::RwBuffer::new(*b"Hello,                   ");
    let mut buffer = pin!(buffer);
    let bytes: Pin<&mut subscribe::Subscriber<_, _>> = pin!(Default::default());
    let timeout: Pin<&mut subscribe::Subscriber<_, _>> = pin!(Default::default());
    console::read(buffer.as_mut().slice().get(7..23).unwrap(), bytes.as_ref())?;
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
    let buffer = buffer.unallow().buffer();
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
    let done: Pin<&mut subscribe::Subscriber<_, _>> = pin!(Default::default());
    let buffer = allow::RoBuffer::new(*buffer);
    let mut buffer = pin!(buffer);
    let to_print = buffer.as_mut().slice().get(0..msg_len).unwrap();
    console::write(to_print, done.as_ref())?;
    loop {
        TockSyscalls::yield_wait();
    }
}
