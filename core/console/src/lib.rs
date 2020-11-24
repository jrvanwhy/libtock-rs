// TODO: This is a work in progress! It is being written by GitHub user
// jrvanwhy. I am still trying to figure out a lot of the details around how
// drivers interface with application code, system calls, and unit tests. As a
// result, there are a lot of TODOs scattered through the code that jrvanwhy
// will work on.

#![no_std]

//! Provides a direct interface to the Tock `console` capsule. The console
//! system call API is documented at
//! https://github.com/tock/tock/blob/master/doc/syscalls/00001_console.md.

use libtock_platform::{FreeCallback, Locator, SubscribeData, SubscribeResponse, Syscalls};

pub struct WriteCompleted<D: SubscribeData> {
    pub bytes: usize,
    pub data: D,
}

pub struct Console<S, L> {
    syscalls: core::marker::PhantomData<S>,
    locator: core::marker::PhantomData<L>,
    written: core::cell::Cell<usize>,
}

impl<S, L> Console<S, L> {
    pub const fn new() -> Self {
        Console { syscalls: core::marker::PhantomData,
                  locator: core::marker::PhantomData,
                  written: core::cell::Cell::new(0) }
    }
}

impl<S: Syscalls, L: Locator<T = Self>> Console<S, L> {
    pub fn set_write_callback<C: FreeCallback<WriteCompleted<D>>, D: SubscribeData>(
        &self, data: D
    ) {
        S::subscribe::<WriteCallback<C, L>, _>(1, 1, data)
    }

    pub fn set_write_buffer(&self, buffer: &'static [u8]) {
        S::const_allow(1, 1, buffer)
    }

    pub fn start_write(&self, bytes: usize) {
        S::command(1, 1, bytes, 0)
    }
}

struct WriteCallback<C, L> {
    _callback: C,
    _locator: L,
}

impl<C: FreeCallback<WriteCompleted<D>>, D: SubscribeData, S: 'static, L: Locator<T = Console<S, L>>>
FreeCallback<SubscribeResponse<D>> for WriteCallback<C, L> {
    fn call(response: SubscribeResponse<D>) {
        L::locate().written.set(L::locate().written.get() + response.arg1);
        C::call(WriteCompleted { bytes: response.arg1, data: response.data });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn write() {
        extern crate std;

        use libtock_sync::SyncAdapter;
        use libtock_unittest::{FakeConsole, FakeSyscalls, test_component};
        use super::{Console, WriteCompleted};

        let fake_syscalls = FakeSyscalls::new();
        let fake_console = FakeConsole::new();
        fake_syscalls.add_driver(fake_console.clone());
        test_component![
            SyncLocator, SYNC_ADAPTER;
            sync_adapter: SyncAdapter<FakeSyscalls, WriteCompleted<usize>> = SyncAdapter::new()
        ];
        test_component![ConsoleLocator, CONSOLE;
                        console: Console<FakeSyscalls, ConsoleLocator> = Console::new()];

        console.set_write_callback::<SyncLocator, _>(1234);
        console.set_write_buffer(b"Hello");
        console.start_write(5);
        let response = sync_adapter.wait();
        assert_eq!(response.bytes, 5);
        assert_eq!(response.data, 1234);
        assert_eq!(fake_console.get_output(), b"Hello");
    }
}
