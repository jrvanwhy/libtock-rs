//! Safe constructors for `libtock_platform::CommandReturn` variants, for use by
//! fake drivers.

use libtock_platform::{return_variant, CommandReturn, ErrorCode};

pub fn failure(error_code: ErrorCode) -> CommandReturn {
    unsafe { CommandReturn::new(return_variant::FAILURE, error_code as usize, 0, 0) }
}

pub fn failure_u32(error_code: ErrorCode, val: u32) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::FAILURE_U32,
            error_code as usize,
            val as usize,
            0,
        )
    }
}

pub fn failure_2_u32(error_code: ErrorCode, val1: u32, val2: u32) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::FAILURE_2_U32,
            error_code as usize,
            val1 as usize,
            val2 as usize,
        )
    }
}

pub fn failure_u64(error_code: ErrorCode, val: u64) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::FAILURE_U64,
            error_code as usize,
            (val & 0xFFFFFFFF) as usize,
            (val >> 32) as usize,
        )
    }
}

pub fn success() -> CommandReturn {
    unsafe { CommandReturn::new(return_variant::SUCCESS, 0, 0, 0) }
}

pub fn success_u32(val: u32) -> CommandReturn {
    unsafe { CommandReturn::new(return_variant::SUCCESS_U32, val as usize, 0, 0) }
}

pub fn success_2_u32(val1: u32, val2: u32) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::SUCCESS_2_U32,
            val1 as usize,
            val2 as usize,
            0,
        )
    }
}

pub fn success_u64(val: u64) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::SUCCESS_U64,
            (val & 0xFFFFFFFF) as usize,
            (val >> 32) as usize,
            0,
        )
    }
}

pub fn success_3_u32(val1: u32, val2: u32, val3: u32) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::SUCCESS_3_U32,
            val1 as usize,
            val2 as usize,
            val3 as usize,
        )
    }
}

pub fn success_u32_u64(val1: u32, val2: u64) -> CommandReturn {
    unsafe {
        CommandReturn::new(
            return_variant::SUCCESS_U32_U64,
            val1 as usize,
            (val2 & 0xFFFFFFFF) as usize,
            (val2 >> 32) as usize,
        )
    }
}
