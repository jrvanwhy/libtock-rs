//! Unit tests for the `subscribe` system call.
use core::cell::Cell;
use libtock_platform::{CallbackContext, FreeCallback, SubscribeResponse, Syscalls};
use libtock_unittest::{fake, SyscallLogEntry};

// Tests `subscribe` with a reference to a cell as the data.
#[test]
fn subscribe_static_cell_ref() {
    struct Callback;
    impl FreeCallback<SubscribeResponse<&'static Cell<u8>>> for Callback {
        fn call(_context: CallbackContext, response: SubscribeResponse<&'static Cell<u8>>) {
            response.data.set(1);
        }
    }

    let kernel = fake::Kernel::new();
    let buffer = Box::leak(Box::new(Cell::new(0)));
    let buffer_raw = buffer as *mut _;
    // TODO: Add a subscribe mock mechanism. Add an assert on the result of this
    // subscribe (it will need to succeed).
    let _todo = fake::Kernel::subscribe_static::<Callback, _>(2, 3, buffer);
    buffer.set(buffer.get() + 1);
    // TODO: Make `yield()` functional, change this test to use a `yield` call.
    let syscall_log = kernel.take_syscall_log();
    if let &[SyscallLogEntry::Subscribe {
        driver_id: 2,
        subscribe: 3,
        callback,
        data,
    }] = syscall_log.as_slice()
    {
        unsafe {
            callback(0, 0, 0, data);
        }
    } else {
        panic!("Unexpected syscall log {:?}", syscall_log);
    }
    buffer.set(buffer.get() + 1);
    unsafe {
        Box::from_raw(buffer_raw);
    }
}

// Tests `subscribe` with a reference as the data.
#[test]
fn subscribe_static_ref() {
    struct Callback;
    impl FreeCallback<SubscribeResponse<&'static u8>> for Callback {
        fn call(_context: CallbackContext, response: SubscribeResponse<&'static u8>) {
            println!("response.data: {}", response.data);
        }
    }

    let kernel = fake::Kernel::new();
    let buffer = Box::leak(Box::new(0));
    let buffer_raw = buffer as *mut _;
    // TODO: Add a subscribe mock mechanism. Add an assert on the result of this
    // subscribe (it will need to succeed).
    let _todo = fake::Kernel::subscribe_static::<Callback, _>(2, 3, buffer);
    println!("buffer value: {}", buffer);
    // TODO: Make `yield()` functional, change this test to use a `yield` call.
    let syscall_log = kernel.take_syscall_log();
    if let &[SyscallLogEntry::Subscribe {
        driver_id: 2,
        subscribe: 3,
        callback,
        data,
    }] = syscall_log.as_slice()
    {
        unsafe {
            callback(0, 0, 0, data);
        }
    } else {
        panic!("Unexpected syscall log {:?}", syscall_log);
    }
    println!("buffer value: {}", buffer);
    unsafe {
        Box::from_raw(buffer_raw);
    }
}
