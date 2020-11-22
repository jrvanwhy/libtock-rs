// TODO: This is a work in progress! It is being written by GitHub user
// jrvanwhy. I am still trying to figure out a lot of the details around how
// drivers interface with application code, system calls, and unit tests. As a
// result, there are a lot of TODOs scattered through the code that jrvanwhy
// will work on.

#![no_std]

//! Provides a direct interface to the Tock `console` capsule. The console
//! system call API is documented at
//! https://github.com/tock/tock/blob/master/doc/syscalls/00001_console.md.

use libtock_platform::{Callback, SubscribeData, SubscribeResponse, Syscalls};

pub struct WriteCompleted<D: SubscribeData> {
    pub bytes: usize,
    pub data: D,
}

#[derive(Copy, Clone)]
pub struct Console<S> {
    syscalls: S,
}

impl<S> Console<S> {
    pub const fn new(syscalls: S) -> Self {
        Console { syscalls }
    }
}

impl<'k, S: Syscalls<'k>> Console<S> {
    pub fn set_write_callback<C: Callback<WriteCompleted<D>> + 'k, D: SubscribeData + 'k>(
        self, callback: C, data: D
    ) {
        self.syscalls.subscribe(1, 1, WriteCallback { callback }, data)
    }

    pub fn set_write_buffer(self, buffer: &'k [u8]) {
        self.syscalls.const_allow(1, 1, buffer)
    }

    pub fn start_write(self, bytes: usize) {
        self.syscalls.command(1, 1, bytes, 0)
    }
}

#[derive(Clone, Copy)]
struct WriteCallback<C: Copy> {
    callback: C,
}

impl<C: Callback<WriteCompleted<D>>, D: SubscribeData> Callback<SubscribeResponse<D>> for WriteCallback<C> {
    fn locate() -> Self { WriteCallback { callback: C::locate() } }

    fn call(self, response: SubscribeResponse<D>) {
        self.callback.call(WriteCompleted { bytes: response.arg1, data: response.data });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn write() {
        extern crate std;

        use libtock_sync::SyncAdapter;
        use libtock_unittest::FakeSyscalls;
        use super::Console;

        let sync_adapter = &SyncAdapter::new();
        let syscalls = &FakeSyscalls::new();
        let console = Console::new(syscalls);

        console.set_write_callback(sync_adapter, 1234);
        console.set_write_buffer(b"Hello");
        console.start_write(5);
        let response = sync_adapter.wait(syscalls);
        assert_eq!(response.bytes, 5);
        assert_eq!(response.data, 1234);
        assert_eq!(syscalls.read_buffer(), b"Hello");
    }
}
