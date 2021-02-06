use std::cell::Cell;
use std::rc::{Rc, Weak};

/// A fake implementation of the Tock system calls. Provides
/// `libtock_platform::Syscalls` by implementing
/// `libtock_platform::RawSyscalls`. Allows `fake::Driver`s to be attached, and
/// routes system calls to the correct fake driver.
///
/// Note that there can only be one `Kernel` instance per thread, as a
/// thread-local variable is used to implement `libtock_platform::RawSyscalls`.
/// As such, test code is given a `Rc<Kernel>` rather than a `Kernel` instance
/// directly.
// TODO: Define the `fake::Driver` trait and add support for fake drivers in
// Kernel.
pub struct Kernel {}

impl Kernel {
    /// Creates a `Kernel` for this thread and returns a reference to it. If
    /// there is already a `Kernel` for this thread, `new` panics.
    pub fn new() -> Rc<Kernel> {
        let rc = Rc::new(Kernel {});
        FAKE.with(|cell| {
            if cell.replace(Rc::downgrade(&rc)).strong_count() != 0 {
                panic!("New Kernel created before the previous one was dropped.");
            }
        });
        rc
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        // Note that Weak::new() does not allocate, whereas Default::Default()
        // does.
        FAKE.with(|cell| cell.replace(Weak::new()));
    }
}

// -----------------------------------------------------------------------------
// Implementation details below.
// -----------------------------------------------------------------------------

// A handle to this thread's Kernel instance. Used by the implementation of
// RawSyscalls on Kernel. This is a weak reference so that when the unit test is
// done with its Kernel, the following cleanup can happen:
//   1. The test drops its Rc<Kernel>
//   2. The strong count drops to 0 so the Kernel is dropped.
//   3. Kernel's Drop implementation clears out FAKE, removing the weak
//      reference.
//   4. The backing storage holding the Kernel is deallocated.
thread_local!(static FAKE: Cell<Weak<Kernel>> = Cell::new(Weak::new()));
