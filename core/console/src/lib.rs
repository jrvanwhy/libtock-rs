// TODO: This is a work in progress! It is being written by GitHub user
// jrvanwhy. I am still trying to figure out a lot of the details around how
// drivers interface with application code, system calls, and unit tests. As a
// result, there are a lot of TODOs scattered through the code that jrvanwhy
// will work on.

#![no_std]

//! Provides a direct interface to the Tock `console` capsule. The console
//! system call API is documented at
//! https://github.com/tock/tock/blob/master/doc/syscalls/00001_console.md.

use libtock_platform::{SubscribeCallback, Syscalls};

pub fn set_write_callback<S: Syscalls, Callback: FreeCallback<WriteCompleted>>(syscalls: S, data: usize) {
    syscalls.subscribe(1, 1, write_complete::<Callback>, data);
}

extern "C" fn write_complete<Callback: FreeCallback<WriteCompleted>>(bytes: usize, _: usize, _: usize, data: usize) {
    Callback::call(WriteCompleted { bytes, data });
}

pub fn set_write_buffer<S: Syscalls>(syscalls: S, buffer: &'static [u8]) {
    unsafe { syscalls.const_allow(1, 1, buffer.as_ptr(), buffer.len()); }
}

pub fn start_write<S: Syscalls>(syscalls: S, bytes: usize) {
    syscalls.command(1, 1, bytes, 0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn write() {
        extern crate std;

        use libtock_platform::MethodCallback;
        use libtock_sync::SyncAdapter;
        use libtock_unittest::FakeSyscalls;
        use std::boxed::Box;
        use std::thread_local;
        use super::{set_write_buffer, set_write_callback, start_write, WriteCompleted};

        let syscalls: &_ = Box::leak(Box::new(FakeSyscalls::new()));
        libtock_unittest::test_component![SyncAdapterLink, sync_adapter: SyncAdapter<WriteCompleted, &'static FakeSyscalls>
                                          = SyncAdapter::new(syscalls)];

        set_write_callback::<_, SyncAdapterLink>(syscalls, 1234);
        set_write_buffer(syscalls, b"Hello");
        start_write(syscalls, 5);
        let response = sync_adapter.wait();
        assert_eq!(response.bytes, 5);
        assert_eq!(response.data, 1234);
        assert_eq!(syscalls.read_buffer(), b"Hello");
    }
}
