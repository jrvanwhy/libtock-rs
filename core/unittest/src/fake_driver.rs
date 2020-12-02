//! Provides the `FakeDriver` trait, which is implemented by fake system call
//! drivers. A test can add `FakeDriver` instances to its `FakeSyscalls`, after
//! which `FakeSyscalls` will system calls to the correct `FakeDriver` instance.

use libtock_platform::{error_code, ErrorCode};

pub trait FakeDriver {
    /// Returns the driver number that this is faking. Should generally return a
    /// constant value. `FakeSyscalls` uses this to route system calls to the
    /// correct Driver instance.
    fn driver_id(&self) -> u32;

    fn subscribe(&self, _subscribe_id: u32) -> Result<(), ErrorCode> {
        Err(error_code::NOSUPPORT)
    }

    fn command(&self, _command_id: u32, _arg0: u32, _arg1: u32) -> CommandResult {
        CommandResult::Failure(error_code::NOSUPPORT)
    }

    fn allow_readwrite(&self, _buffer_id: u32) -> Result<(), ErrorCode> {
        Err(error_code::NOSUPPORT)
    }

    fn allow_readonly(&self, _buffer_id: u32) -> Result<(), ErrorCode> {
        Err(error_code::NOSUPPORT)
    }
}

/// Represents any valid return type from a command call.
pub enum CommandResult {
    Failure(ErrorCode),
    FailureU32(ErrorCode, u32),
    FailureU32U32(ErrorCode, u32, u32),
    FailureU64(ErrorCode, u64),
    Success,
    SuccessU32(u32),
    SuccessU32U32(u32, u32),
    SuccessU64(u64),
    SuccessU32U32U32(u32, u32, u32),
    SuccessU64U32(u64, u32),
}
