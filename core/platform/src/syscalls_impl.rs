//! Implements `Syscalls` for all types that implement `RawSyscalls`.

use crate::{
    return_variant, CallbackContext, CommandReturn, ErrorCode, FreeCallback, RawSyscalls,
    SubscribeData, SubscribeResponse, Syscalls, YieldNoWaitReturn,
};

mod class_id {
    pub const SUBSCRIBE: u32 = 1;
    pub const COMMAND: u32 = 2;
    pub const ALLOW_RW: u32 = 3;
}

mod yield_op {
    pub const NO_WAIT: u32 = 0;
    pub const WAIT: u32 = 1;
}

impl<S: RawSyscalls> Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_no_wait() -> YieldNoWaitReturn {
        unsafe {
            // This can be uninitialized because yield2's documentation says it
            // is designed to accept an uninitialized flag. In particular, its
            // value is not read before it is set.
            let mut flag = core::mem::MaybeUninit::uninit();

            // Flag is valid to write a YieldNoWaitReturn to.
            Self::yield2(yield_op::NO_WAIT, flag.as_mut_ptr());

            // yield2 guarantees it sets (initializes) flag before returning, so
            // this is sound.
            flag.assume_init()
        }
    }

    fn yield_wait() {
        unsafe {
            Self::yield1(yield_op::WAIT);
        }
    }

    // -------------------------------------------------------------------------
    // Subscribe
    // -------------------------------------------------------------------------

    fn subscribe_static<
        C: FreeCallback<SubscribeResponse<D>>,
        D: 'static + Copy + SubscribeData,
    >(
        driver_id: u32,
        subscribe_id: u32,
        data: D,
    ) -> Result<(), ErrorCode> {
        unsafe {
            let (r0, r1, _r2, _r3) = Self::syscall4::<{ class_id::SUBSCRIBE }>(
                driver_id,
                subscribe_id as usize,
                callback_static::<C, D> as usize,
                data.to_usize(),
            );
            if r0 == return_variant::FAILURE_2_U32.into() {
                return Err(core::mem::transmute(r1 as u32));
            }
            Ok(())
        }
    }

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    fn command(driver_id: u32, command_id: u32, argument1: u32, argument2: u32) -> CommandReturn {
        unsafe {
            // A `command` system call cannot violate memory safety on its own.
            let (r0, r1, r2, r3) = Self::syscall4::<{ class_id::COMMAND }>(
                driver_id,
                command_id as usize,
                argument1 as usize,
                argument2 as usize,
            );
            // r0 and r1 are taken directly from the kernel's response, and
            // therefore respect the "r1 must be an error code if r0 is a
            // failure variant" requirement.
            CommandReturn::new(r0.into(), r1, r2, r3)
        }
    }

    // -------------------------------------------------------------------------
    // Allow Read-Write
    // -------------------------------------------------------------------------

    fn allow_rw_static(driver_id: u32, buffer_id: u32, buffer: &'static mut [u8])
        -> Result<&'static mut [u8], (ErrorCode, &'static mut [u8])>
    {
        (|buffer: [&'static mut [u8]; 1]| {
            let buffer_address = buffer[0].as_mut_ptr();
            let buffer_len = buffer[0].len();
            drop(buffer);
            unsafe { core::slice::from_raw_parts_mut(buffer_address, buffer_len); }
            let (r0, r1, r2, r3) = unsafe { Self::syscall4::<{ class_id::ALLOW_RW }>(
                driver_id,
                buffer_id as usize,
                buffer_address as usize,
                buffer_len,
            )};
            if r0 == return_variant::FAILURE_2_U32.into() {
                let error_code = unsafe { core::mem::transmute(r1 as u32) };
                // Because the slice is just buffer returned, we know r2 is
                // non-null.
                let slice = unsafe { core::slice::from_raw_parts_mut(r2 as *mut u8, r3) };
                return Err((error_code, slice));
            }
            Ok(unsafe { allow_return_to_slice_mut(r1, r2) })
        })([buffer])
    }
}

// Converts the address and length returned by the allow system calls into a
// mutable reference. This handles the case where the returned address is 0,
// which should be mapped to an empty slice.
unsafe fn allow_return_to_slice_mut<'b>(address: usize, len: usize) -> &'b mut [u8] {
    use core::ptr::NonNull;
    let data = if address == 0 { NonNull::dangling().as_ptr() } else { address as *mut _ };
    core::slice::from_raw_parts_mut(data, len)
}

unsafe extern "C" fn callback_static<
    C: FreeCallback<SubscribeResponse<D>>,
    D: 'static + Copy + SubscribeData,
>(
    arg1: u32,
    arg2: u32,
    arg3: u32,
    data: usize,
) {
    use core::marker::PhantomData;
    C::call(
        CallbackContext {
            _phantom: PhantomData,
        },
        SubscribeResponse {
            arg1,
            arg2,
            arg3,
            data: D::from_usize(data),
        },
    );
}
