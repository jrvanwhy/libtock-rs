//! Demonstrates asynchronous usage of the console driver.

#![no_std]

extern crate libtock_core;

#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];

use libtock_console::{set_write_buffer, set_write_callback, start_write, WriteCompleted};
use libtock_platform::{Callback, Syscalls};
use libtock_runtime::TockSyscalls;

libtock_runtime::static_component![AppLink, APP: App = App::new()];
libtock_runtime::static_component![ConsoleLink, CONSOLE: Console<TockSyscalls> = Console::new(TockSyscalls)];

static mut GREETING: [u8; 7] = *b"Hello, ";
static mut NOUN: [u8; 7] = *b"World!\n";

fn main() {
    CONSOLE.set_write_callback(AppLink, 0);
    CONSOLE.set_write_buffer(unsafe { &GREETING } );
    CONSOLE.start_write(unsafe { GREETING.len() });
    loop {
        TockSyscalls.yieldk();
    }
}

struct App {
    done: core::cell::Cell<bool>
}

impl App {
    pub const fn new() -> App {
        App {
            done: core::cell::Cell::new(false)
        }
    }
}

impl Callback<WriteCompleted<usize>> for &App {
    fn locate() -> Self { panic!("App's callback should not be used directly"); }

    fn call(self, _response: WriteCompleted<usize>) {
        if self.done.get() { return; }
        self.done.set(true);
        CONSOLE.set_write_buffer(unsafe { &NOUN } );
        CONSOLE.start_write(unsafe { NOUN.len() } );
    }
}
