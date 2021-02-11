/// SyscallLogEntry represents a system call made during test execution.
#[derive(Debug, PartialEq)]
pub enum SyscallLogEntry {
    YieldWait,
    YieldNoWait,
    Subscribe {
        driver_id: u32,
        subscribe: u32,
        callback: unsafe extern "C" fn(u32, u32, u32, usize),
        data: usize,
    },
    Command {
        driver_id: u32,
        command: u32,
        arg1: u32,
        arg2: u32,
    },
    AllowRw {
        driver_id: u32,
        buffer_id: u32,
    },
}
