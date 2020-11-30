// TODO: Implement `libtock_runtime` and `libtock_unittest`, which are
// referenced in the comment on `Syscalls`.

use crate::{
    CommandReturn, ErrorCode, FreeCallback, SubscribeData, SubscribeResponse, Subscription,
};

/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::FakeSyscalls` (by way of `RawSyscalls`).
pub trait Syscalls {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait();

    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending. Returns true if a callback was executed, false otherwise.
    fn yield_no_wait() -> bool;

    // -------------------------------------------------------------------------
    // Subscribe
    // -------------------------------------------------------------------------

    /// Subscribe a callback to a given subscribe ID. The given data will be
    /// passed to each invocation of the specified callback.
    fn subscribe<C: FreeCallback<SubscribeResponse<D>>, D: 'static + SubscribeData + Copy>(
        driver: usize,
        subscribe_id: usize,
        data: D,
    ) -> Result<(), ErrorCode>;

    /// Temporarily subscribes `callback` to the given subscription, runs
    /// `body`, then unsubscribes `callback`. Requires a `HaltBehavior` to
    /// handle the case where the `unsubscribe` fails, because `callback` does
    /// not need to be `'static`.
    fn subscribe_nonstatic<C: Fn(usize, usize, usize), B: FnOnce(), HB: HaltBehavior>(
        driver: usize,
        subscribe_id: usize,
        callback: &C,
        body: B,
    ) -> Result<(), ErrorCode>;

    /// Subscribe a callback to a given subscribe ID, which will automatically
    /// be unsubscribed the first time it is invoked. This allows us to pass
    /// non-`Copy` data to a callback.
    fn subscribe_once<
        C: FreeCallback<SubscribeResponse<D>>,
        D: 'static + SubscribeData,
        SId: Subscription,
        HB: HaltBehavior,
    >(
        data: D,
    ) -> Result<(), ErrorCode>;

    /// Un-subscribe from the given subscribe ID, preventing further invocations
    /// of the currently-subscribed callback.
    fn unsubscribe(driver: usize, subscribe_id: usize) -> Result<(), ErrorCode>;

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    /// Invokes the specified command.
    fn command(
        driver: usize,
        command_id: usize,
        argument1: usize,
        argument2: usize,
    ) -> CommandReturn;

    // -------------------------------------------------------------------------
    // Read-Write Allow
    // -------------------------------------------------------------------------

    /// Shares a buffer with the specified `driver` under the given `buffer_id`.
    /// If successful, returns the buffer previously stored in that combination
    /// of `driver` and `buffer_id`. If it fails, it returns the error code and
    /// the buffer that was passed in. If there was no previously-stored buffer,
    /// returns a zero-sized buffer.
    fn allow_mut(
        driver: usize,
        buffer_id: usize,
        buffer: &'static mut [u8],
    ) -> Result<&'static mut [u8], AllowMutError>;

    // -------------------------------------------------------------------------
    // Read-Only Allow
    // -------------------------------------------------------------------------

    /// Shares a buffer with the specified `driver` under the given `buffer_id`.
    fn allow_ro(driver: usize, buffer_id: usize, buffer: &'static [u8]) -> Result<(), ErrorCode>;

    // -------------------------------------------------------------------------
    // Memop
    // -------------------------------------------------------------------------

    /// Sets the program break to the given value.
    fn memop_brk(new_break: usize) -> Result<(), ErrorCode>;

    /// Moves the program break by the given amount. If successful, returns the
    /// previous program break.
    fn memop_sbrk(delta_break: isize) -> Result<usize, ErrorCode>;

    /// Returns the address of the start of the process' RAM allocation.
    fn memop_memory_start() -> Result<*const (), ErrorCode>;

    /// Returns the first address after the end of the process' RAM allocation.
    fn memop_memory_end() -> Result<*const (), ErrorCode>;

    /// Returns the start of the process' flash region, where the TBF header is
    /// located.
    fn memop_flash_start() -> Result<*const (), ErrorCode>;

    /// Returns the first address after the process' flash region.
    fn memop_flash_end() -> Result<*const (), ErrorCode>;

    /// Returns the lowest address of the grant region.
    fn memop_grant_start() -> Result<*const (), ErrorCode>;

    /// Returns the number of writable flash region this process has.
    fn memop_flash_regions() -> Result<usize, ErrorCode>;

    /// Get the start address of the specified flash region.
    fn memop_flash_region_start(region: usize) -> Result<*const (), ErrorCode>;

    /// Get the first address after the end of the specified flash region.
    fn memop_flash_region_end(region: usize) -> Result<*const (), ErrorCode>;

    /// Specify the top of the application stack (for debugging purposes).
    fn memop_specify_stack_top(stack_top: *const ()) -> Result<(), ErrorCode>;

    /// Specify the start of the application heap (for debugging purposes).
    fn memop_specify_heap_start(heap_start: *const ()) -> Result<(), ErrorCode>;
}

/// The error type returned by `allow_mut`.
pub struct AllowMutError {
    pub buffer: &'static mut [u8],
    pub error_code: ErrorCode,
}

/// Trait representing a possible behavior for when `Syscalls` is unable to
/// continue execution safely. For example, `HaltBehavior::halt` is invoked if
/// `subscribe_once` is unable to unsubscribe its callback, which is required
/// for memory safety.
pub trait HaltBehavior {
    fn halt() -> !;
}
