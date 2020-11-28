//! Implements `Syscalls` for all types that implement `RawSyscalls`.

use crate::{
    error_code, return_type, AllowMutError, CallbackContext, CommandReturn, ErrorCode,
    FreeCallback, HaltBehavior, OneArgMemop, RawSyscalls, SubscribeData, SubscribeResponse,
    Subscription, Syscalls, YieldType, ZeroArgMemop,
};
use core::slice::from_raw_parts_mut;

impl<S: RawSyscalls> Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_wait() {
        Self::raw_yield(YieldType::Wait);
    }

    fn yield_no_wait() -> bool {
        Self::raw_yield(YieldType::NoWait) != return_type::FAILURE.into()
    }

    // -------------------------------------------------------------------------
    // Subscribe
    // -------------------------------------------------------------------------

    fn subscribe<C: FreeCallback<SubscribeResponse<D>>, D: 'static + SubscribeData + Copy>(
        driver: usize,
        subscribe_id: usize,
        data: D,
    ) -> Result<(), ErrorCode> {
        let (return_type, fail_error_code, _, _) = unsafe {
            Self::four_arg_syscall(
                driver,
                subscribe_id,
                subscribe_callback::<C, D> as usize,
                data.to_usize(),
                syscall_class::SUBSCRIBE,
            )
        };
        match return_type.into() {
            return_type::SUCCESS_2_U32 => Ok(()),
            return_type::FAILURE_2_U32 => Err(fail_error_code.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn subscribe_nonstatic<C: Fn(usize, usize, usize), B: FnOnce(), HB: HaltBehavior>(
        driver: usize,
        subscribe_id: usize,
        callback: &C,
        body: B,
    ) -> Result<(), ErrorCode> {
        let (return_type, fail_error_code, _, _) = unsafe {
            Self::four_arg_syscall(
                driver,
                subscribe_id,
                subscribe_nonstatic_callback::<C> as usize,
                callback.to_usize(),
                syscall_class::SUBSCRIBE,
            )
        };
        match return_type.into() {
            return_type::SUCCESS_2_U32 => {}
            return_type::FAILURE_2_U32 => return Err(fail_error_code.into()),
            _ => return Err(error_code::BADRVAL),
        }
        body();
        if Self::unsubscribe(driver, subscribe_id).is_err() {
            HB::halt();
        }
        Ok(())
    }

    fn subscribe_once<
        C: FreeCallback<SubscribeResponse<D>>,
        D: 'static + SubscribeData,
        SId: Subscription,
        HB: HaltBehavior,
    >(
        data: D,
    ) -> Result<(), ErrorCode> {
        let (return_type, fail_error_code, _, _) = unsafe {
            Self::four_arg_syscall(
                SId::DRIVER,
                SId::ID,
                subscribe_once_callback::<C, D, SId, HB, Self> as usize,
                data.to_usize(),
                syscall_class::SUBSCRIBE,
            )
        };
        match return_type.into() {
            return_type::FAILURE_2_U32 => Err(fail_error_code.into()),
            return_type::SUCCESS_2_U32 => Ok(()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn unsubscribe(driver: usize, subscribe_id: usize) -> Result<(), ErrorCode> {
        let (return_type, fail_error_code, _, _) =
            unsafe { Self::four_arg_syscall(driver, subscribe_id, 0, 0, syscall_class::SUBSCRIBE) };
        match return_type.into() {
            return_type::FAILURE_2_U32 => Err(fail_error_code.into()),
            return_type::SUCCESS_2_U32 => Ok(()),
            _ => Err(error_code::BADRVAL),
        }
    }

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    fn command(
        driver: usize,
        command_id: usize,
        argument1: usize,
        argument2: usize,
    ) -> CommandReturn {
        let (return_type, r1, r2, r3) = unsafe {
            Self::four_arg_syscall(
                driver,
                command_id,
                argument1,
                argument2,
                syscall_class::COMMAND,
            )
        };
        CommandReturn {
            return_type: return_type.into(),
            r1,
            r2,
            r3,
        }
    }

    // -------------------------------------------------------------------------
    // Read-Write Allow
    // -------------------------------------------------------------------------

    fn allow_mut(
        driver: usize,
        buffer_id: usize,
        buffer: &'static mut [u8],
    ) -> Result<&'static mut [u8], AllowMutError> {
        // Disassemble buffer and drop it. We can't keep buffer around after the
        // allow call because having a &mut [u8] to memory the kernel can mutate
        // will cause undefined behavior.
        let buffer_ptr = buffer.as_mut_ptr() as usize;
        let buffer_len = buffer.len();
        // `clippy` claims this drop does nothing, but that's false, because
        // it avoids undefined behavior (see previous comment).
        #[allow(clippy::drop_ref)]
        drop(buffer);
        let (return_type, r1, r2, fail_len) = unsafe {
            Self::four_arg_syscall(
                driver,
                buffer_id,
                buffer_ptr,
                buffer_len,
                syscall_class::RW_ALLOW,
            )
        };
        match return_type.into() {
            return_type::FAILURE_2_U32 => Err(AllowMutError {
                buffer: unsafe { from_raw_parts_mut(r2 as *mut u8, fail_len) },
                error_code: r1.into(),
            }),
            return_type::SUCCESS_2_U32 => Ok(unsafe { from_raw_parts_mut(r1 as *mut u8, r2) }),
            // In the event of an unrecognized return type, we cannot be sure
            // what happened to buffer. Sadly the buffer is probably lost (and
            // if this buffer isn't lost, the existing buffer is lost...).
            // Return a zero-sized buffer.
            _ => Err(AllowMutError {
                buffer: &mut [],
                error_code: error_code::BADRVAL,
            }),
        }
    }

    // -------------------------------------------------------------------------
    // Read-Only Allow
    // -------------------------------------------------------------------------

    /// Shares a buffer with the specified `driver` under the given `buffer_id`.
    fn allow_ro(driver: usize, buffer_id: usize, buffer: &'static [u8]) -> Result<(), ErrorCode> {
        let (return_type, fail_error_code, _, _) = unsafe {
            Self::four_arg_syscall(
                driver,
                buffer_id,
                buffer.as_ptr() as usize,
                buffer.len(),
                syscall_class::RO_ALLOW,
            )
        };
        match return_type.into() {
            return_type::FAILURE_2_U32 => Err(fail_error_code.into()),
            return_type::SUCCESS_2_U32 => Ok(()),
            _ => Err(error_code::BADRVAL),
        }
    }

    // -------------------------------------------------------------------------
    // Memop
    // -------------------------------------------------------------------------

    fn memop_brk(new_break: usize) -> Result<(), ErrorCode> {
        let (return_type, fail_error_code) = Self::one_arg_memop(OneArgMemop::Brk, new_break);
        match return_type.into() {
            return_type::SUCCESS => Ok(()),
            return_type::FAILURE => Err(fail_error_code.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_sbrk(delta_break: isize) -> Result<usize, ErrorCode> {
        let (return_type, r1) = Self::one_arg_memop(OneArgMemop::Sbrk, delta_break as usize);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_memory_start() -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::zero_arg_memop(ZeroArgMemop::MemoryStart);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_memory_end() -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::zero_arg_memop(ZeroArgMemop::MemoryEnd);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_flash_start() -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::zero_arg_memop(ZeroArgMemop::FlashStart);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_flash_end() -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::zero_arg_memop(ZeroArgMemop::FlashEnd);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_grant_start() -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::zero_arg_memop(ZeroArgMemop::GrantStart);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_flash_regions() -> Result<usize, ErrorCode> {
        let (return_type, r1) = Self::zero_arg_memop(ZeroArgMemop::FlashRegions);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_flash_region_start(region: usize) -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::one_arg_memop(OneArgMemop::FlashRegionStart, region);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_flash_region_end(region: usize) -> Result<*const (), ErrorCode> {
        let (return_type, r1) = Self::one_arg_memop(OneArgMemop::FlashRegionEnd, region);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(r1 as *const ()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_specify_stack_top(stack_top: *const ()) -> Result<(), ErrorCode> {
        let (return_type, r1) =
            Self::one_arg_memop(OneArgMemop::SpecifyStackTop, stack_top as usize);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }

    fn memop_specify_heap_start(heap_start: *const ()) -> Result<(), ErrorCode> {
        let (return_type, r1) =
            Self::one_arg_memop(OneArgMemop::SpecifyHeapStart, heap_start as usize);
        match return_type.into() {
            return_type::SUCCESS_U32 => Ok(()),
            return_type::FAILURE => Err(r1.into()),
            _ => Err(error_code::BADRVAL),
        }
    }
}

unsafe extern "C" fn subscribe_callback<C: FreeCallback<SubscribeResponse<D>>, D: SubscribeData>(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    data: usize,
) {
    C::call(
        CallbackContext::new(),
        SubscribeResponse {
            arg1,
            arg2,
            arg3,
            data: D::from_usize(data),
        },
    );
}

unsafe extern "C" fn subscribe_nonstatic_callback<C: Fn(usize, usize, usize)>(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    data: C,
) {
    data(arg1, arg2, arg3);
}

unsafe extern "C" fn subscribe_once_callback<
    C: FreeCallback<SubscribeResponse<D>>,
    D: SubscribeData,
    SId: Subscription,
    HB: HaltBehavior,
    SC: Syscalls,
>(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    data: usize,
) {
    if SC::unsubscribe(SId::DRIVER, SId::ID).is_err() {
        HB::halt();
    }
    C::call(
        CallbackContext::new(),
        SubscribeResponse {
            arg1,
            arg2,
            arg3,
            data: D::from_usize(data),
        },
    );
}

mod syscall_class {
    pub(super) const SUBSCRIBE: u8 = 1;
    pub(super) const COMMAND: u8 = 2;
    pub(super) const RW_ALLOW: u8 = 3;
    pub(super) const RO_ALLOW: u8 = 4;
}
