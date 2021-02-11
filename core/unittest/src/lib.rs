//! Provides testing support needed by `libtock-rs` unit tests and unit tests of
//! code built on `libtock-rs`.

pub mod command_return;
mod kernel;
mod mock;
mod syscall_log;

/// `fake` contains fake implementations of Tock kernel components. Fake
/// components emulate the behavior of the real Tock kernel components, but in
/// the unit test environment. They generally have additional testing features,
/// such as error injection functionality.
///
/// These components are exposed under the `fake` module because otherwise their
/// names would collide with the corresponding drivers (e.g. the fake Console
/// would collide with the Console driver in unit tests). Tests should generally
/// `use libtock_unittest::fake` and refer to the type with the `fake::` prefix
/// (e.g. `fake::Console`).
pub mod fake {
    pub use crate::kernel::Kernel;
}

pub use mock::Mock;
pub use syscall_log::SyscallLogEntry;
