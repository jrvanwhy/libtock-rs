//! Unit tests for the command implementation in Syscalls.
use libtock_platform::{ErrorCode, Syscalls};
use libtock_unittest::{command_return, fake, Mock, SyscallLogEntry};

#[test]
fn command() {
    let kernel = fake::Kernel::new();
    kernel.push_mock(Mock::Command {
        driver_id: 1,
        command: 2,
        arg1: 3,
        arg2: 4,
        output: command_return::failure_u64(ErrorCode::Invalid, 0xAAAAAAAABBBBBBBB),
    });
    assert_eq!(
        fake::Kernel::command(1, 2, 3, 4).get_failure_u64(),
        Some((ErrorCode::Invalid, 0xAAAAAAAABBBBBBBB))
    );
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Command {
            driver_id: 1,
            command: 2,
            arg1: 3,
            arg2: 4
        }]
    );
}
