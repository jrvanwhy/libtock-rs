//! Demonstrates asynchronous usage of the console driver.

#![no_std]

extern crate libtock_core;

#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];

use libtock_console::{set_write_buffer, set_write_callback, start_write, WriteCompleted};
use libtock_platform::{FreeCallback, Syscalls};
use libtock_runtime::TockSyscalls;

static mut GREETING: [u8; 7] = *b"Hello, ";
static mut NOUN: [u8; 7] = *b"World!\n";
static mut DONE: bool = false;

fn main() {
    set_write_callback::<_, App>(TockSyscalls, 0);
    set_write_buffer(TockSyscalls, unsafe { &GREETING } );
    start_write(TockSyscalls, unsafe { GREETING.len() });
    loop {
        TockSyscalls.yieldk();
    }
}

struct App;

impl FreeCallback<WriteCompleted> for App {
    fn call(_response: WriteCompleted) {
        unsafe {
            if DONE { return; }
            DONE = true;
        }
        set_write_buffer(TockSyscalls, unsafe { &NOUN } );
        start_write(TockSyscalls, unsafe { NOUN.len() } );
    }
}
