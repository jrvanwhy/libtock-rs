//! Provides testing support needed by `libtock-rs` unit tests and unit tests of
//! code built on `libtock-rs`.

mod fake_driver;
mod fake_syscalls;

pub use fake_driver::FakeDriver;
pub use fake_syscalls::FakeSyscalls;
