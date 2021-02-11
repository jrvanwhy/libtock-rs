use libtock_platform::{ErrorCode, RawSyscalls, return_variant, YieldNoWaitReturn};

mod class_id {
    pub const SUBSCRIBE: u32 = 1;
    pub const COMMAND: u32 = 2;
    pub const RW_ALLOW: u32 = 3;
    pub const RO_ALLOW: u32 = 4;
}

mod yield_op {
    pub const NO_WAIT: u32 = 0;
    pub const WAIT: u32 = 1;
}

// The main body of yield1. This was separated out because it does not need
// `unsafe`.
fn yield1_impl(op: u32) {
    let kernel = super::get_kernel("raw_yield_wait");
    if op != yield_op::WAIT {
        panic!("Unknown 1-argument yield call invoked: op = {}", op);
    }
    kernel.log_syscall(crate::SyscallLogEntry::YieldWait);
    match kernel.pop_mock() {
        None => {}
        Some(crate::Mock::YieldWait) => {}
        Some(mock) => mock.panic_wrong_call("yield-wait"),
    }

    // TODO: Add driver support, including a callback queue.
}

// The main body of 2. This was factored out of yield2 because only the final
// pointer write requires unsafe, and yield2 is an unsafe function.
fn yield2_impl(op: u32) -> YieldNoWaitReturn {
    let kernel = super::get_kernel("raw_yield_wait");
    if op != yield_op::NO_WAIT {
        panic!("Unknown 2-argument yield call invoked: op = {}", op);
    }
    kernel.log_syscall(crate::SyscallLogEntry::YieldNoWait);
    let mock_callback_ran = match kernel.pop_mock() {
        None => None,
        Some(crate::Mock::YieldNoWait { callback_ran }) => Some(callback_ran),
        Some(mock) => mock.panic_wrong_call("yield-no-wait"),
    };

    // TODO: Add driver support, including a callback queue. For now, just
    // indicate no callback ran.
    let callback_ran = false;

    // Override the output value if a mock was present.
    match mock_callback_ran.unwrap_or(callback_ran) {
        false => YieldNoWaitReturn::NoCallback,
        true => YieldNoWaitReturn::Callback,
    }
}

fn subscribe(driver_id: u32, r1: usize, r2: usize, r3: usize) -> (u32, usize, usize, usize) {
    use std::convert::TryInto;
    let kernel = super::get_kernel("subscribe");
    let subscribe = r1.try_into().expect("Out-of-range subscribe ID passed.");
    let callback = unsafe { std::mem::transmute(r2) };
    let data = r3;
    kernel.log_syscall(crate::SyscallLogEntry::Subscribe {
        driver_id,
        subscribe,
        callback,
        data,
    });
    // TODO: Add mocking for subscribe calls.
    // TODO: Add driver support.
    (
        return_variant::FAILURE_2_U32.into(),
        ErrorCode::NoSupport as usize,
        0,
        0,
    )
}

fn command(driver_id: u32, r1: usize, r2: usize, r3: usize) -> (u32, usize, usize, usize) {
    use std::convert::TryInto;
    let kernel = super::get_kernel("command");
    let command = r1.try_into().expect("Out-of-range command ID passed.");
    let arg1 = r2.try_into().expect("Out-of-range arg1 passed to command.");
    let arg2 = r3.try_into().expect("Out-of-range arg2 passed to command.");
    kernel.log_syscall(crate::SyscallLogEntry::Command {
        driver_id,
        command,
        arg1,
        arg2,
    });
    let mock_output = match kernel.pop_mock() {
        None => None,
        Some(crate::Mock::Command {
            driver_id: mock_driver_id,
            command: mock_command,
            arg1: mock_arg1,
            arg2: mock_arg2,
            output: mock_output,
        }) if mock_driver_id == driver_id
            && mock_command == command
            && mock_arg1 == arg1
            && mock_arg2 == arg2 =>
        {
            Some(mock_output)
        }
        Some(mock) => mock.panic_wrong_call(&format!(
            "command({}, {}, {}, {})",
            driver_id, command, arg1, arg2
        )),
    };

    // TODO: Add driver support, and call the driver here.
    let driver_return = crate::command_return::failure(ErrorCode::NoSupport);

    let command_return = mock_output.unwrap_or(driver_return);
    let (r0, variant_r1, variant_r2, variant_r3) = command_return.raw_registers();
    let (r1, r2, r3) = match command_return.return_variant() {
        return_variant::FAILURE => (variant_r1, r2, r3),
        return_variant::FAILURE_U32 => (variant_r1, variant_r2, r3),
        return_variant::FAILURE_2_U32 => (variant_r1, variant_r2, variant_r3),
        return_variant::FAILURE_U64 => (variant_r1, variant_r2, variant_r3),
        return_variant::SUCCESS => (r1, r2, r3),
        return_variant::SUCCESS_U32 => (variant_r1, r2, r3),
        return_variant::SUCCESS_2_U32 => (variant_r1, variant_r2, r3),
        return_variant::SUCCESS_U64 => (variant_r1, variant_r2, r3),
        return_variant::SUCCESS_3_U32 => (variant_r1, variant_r2, variant_r3),
        return_variant::SUCCESS_U32_U64 => (variant_r1, variant_r2, variant_r3),
        _ => (variant_r1, variant_r2, variant_r3),
    };
    (r0, r1, r2, r3)
}

fn allow_rw(driver_id: u32, r1: usize, r2: usize, r3: usize) -> (u32, usize, usize, usize) {
    use std::convert::TryInto;
    let kernel = super::get_kernel("subscribe");
    let buffer_id = r1.try_into().expect("Out-of-range buffer ID passed.");
    let buffer = unsafe { core::slice::from_raw_parts_mut(r2 as *mut u8, r3) };
    kernel.log_syscall(crate::SyscallLogEntry::AllowRw {
        driver_id,
        buffer_id,
    });
    let mock_output = match kernel.pop_mock() {
        None => None,
        Some(crate::Mock::AllowRw {
            driver_id: mock_driver_id,
            buffer_id: mock_buffer_id,
            output: mock_output,
        }) if mock_driver_id == driver_id
            && mock_buffer_id == buffer_id =>
        {
            Some(mock_output)
        }
        Some(mock) => mock.panic_wrong_call(&format!(
            "allow_rw({}, {}, {:?})",
            driver_id, buffer_id, buffer
        )),
    };

    // TODO: Add driver support.
    let driver_return = Err(ErrorCode::NoSupport);

    match mock_output.unwrap_or(driver_return) {
        Ok(buffer) => (return_variant::SUCCESS_2_U32.into(), buffer.as_ptr() as usize, buffer.len(), r3),
        Err(error_code) => (return_variant::FAILURE_2_U32.into(), error_code as usize, buffer.as_ptr() as usize, buffer.len()),
    }
}

impl RawSyscalls for super::Kernel {
    unsafe fn yield1(op: u32) {
        yield1_impl(op);
    }

    unsafe fn yield2(op: u32, flag: *mut YieldNoWaitReturn) {
        core::ptr::write(flag, yield2_impl(op));
    }

    unsafe fn syscall1<const CLASS: u32>(_r0: u32) -> (u32, usize) {
        unimplemented!();
    }

    unsafe fn syscall2<const CLASS: u32>(_r0: u32, _r1: usize) -> (u32, usize) {
        unimplemented!();
    }

    unsafe fn syscall4<const CLASS: u32>(
        r0: u32,
        r1: usize,
        r2: usize,
        r3: usize,
    ) -> (u32, usize, usize, usize) {
        match CLASS {
            class_id::SUBSCRIBE => subscribe(r0, r1, r2, r3),
            class_id::COMMAND => command(r0, r1, r2, r3),
            class_id::RW_ALLOW => allow_rw(r0, r1, r2, r3),
            class_id::RO_ALLOW => unimplemented!(),
            _ => panic!("Unknown syscall4 call. Class: {}", CLASS),
        }
    }
}
