/// Unit tests for the yield system call implementations in Syscalls.
use libtock_platform::{Syscalls, YieldNoWaitReturn};
use libtock_unittest::{fake, Mock, SyscallLogEntry};

#[test]
fn wait() {
    let kernel = fake::Kernel::new();
    fake::Kernel::yield_wait();
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);
}

// Tests yield_no_wait with no callback invoked.
#[test]
fn no_wait_no_callback() {
    let kernel = fake::Kernel::new();
    assert!(
        fake::Kernel::yield_no_wait() == YieldNoWaitReturn::NoCallback,
        "yield_no_wait() should equal NoCallback, it did not"
    );
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
}

// Tests yield_no_wait with a callback.
#[test]
fn no_wait_callback() {
    let kernel = fake::Kernel::new();
    kernel.push_mock(Mock::YieldNoWait { callback_ran: true });
    assert!(
        fake::Kernel::yield_no_wait() == YieldNoWaitReturn::Callback,
        "yield_no_wait() should equal Callback, it did not"
    );
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
}
