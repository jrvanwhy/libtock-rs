// TODO: Implement `libtock_runtime` and `libtock_unittest`, which are
// referenced in the comment on `Syscalls`.

use crate::{
    CommandReturn, ErrorCode, FreeCallback, SubscribeData, SubscribeResponse, YieldNoWaitReturn,
};

/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::FakeSyscalls` (by way of `RawSyscalls`).
pub trait Syscalls {
    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait();

    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending. Returns true if a callback was executed, false otherwise.
    fn yield_no_wait() -> YieldNoWaitReturn;

    fn subscribe_static<C: FreeCallback<SubscribeResponse<D>>, D: 'static + Copy + SubscribeData>(
        driver_id: u32,
        subscribe_id: u32,
        data: D,
    ) -> Result<(), ErrorCode>;

    fn command(driver_id: u32, command_id: u32, argument1: u32, argument2: u32) -> CommandReturn;

    // TODO: Add a read-write allow interface.
    fn allow_rw_static(driver_id: u32, buffer_id: u32, buffer: &'static mut [u8])
        -> Result<&'static mut [u8], (ErrorCode, &'static mut [u8])>;

    // TODO: Add a read-only allow interface.

    // TODO: Add memop() methods.
}
