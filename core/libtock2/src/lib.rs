#![no_std]

extern crate libtock_panic_debug;

pub use libtock_platform as platform;
pub use libtock_runtime as runtime;

pub mod low_level_debug {
    pub type LowLevelDebug = libtock_low_level_debug::LowLevelDebug<libtock_runtime::TockSyscalls>;
}
