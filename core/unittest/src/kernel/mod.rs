use std::cell::Cell;
use std::rc::{Rc, Weak};

mod raw_syscalls_impl;

/// A fake implementation of the Tock system calls. Provides
/// `libtock_platform::Syscalls` by implementing
/// `libtock_platform::RawSyscalls`. Allows `fake::Driver`s to be attached, and
/// routes system calls to the correct fake driver.
///
/// Note that there can only be one `Kernel` instance per thread, as a
/// thread-local variable is used to implement `libtock_platform::RawSyscalls`.
/// As such, test code is given a `Rc<Kernel>` rather than a `Kernel` instance
/// directly. Because `Rc` is a shared reference, Kernel uses internal
/// mutability extensively.
// TODO: Define the `fake::Driver` trait and add support for fake drivers in
// Kernel.
pub struct Kernel {
    mock_queue: Cell<std::collections::VecDeque<crate::Mock>>,
    syscall_log: Cell<Vec<crate::SyscallLogEntry>>,
}

impl Kernel {
    /// Creates a `Kernel` for this thread and returns a reference to it. If
    /// there is already a `Kernel` for this thread, `new` panics.
    pub fn new() -> Rc<Kernel> {
        let rc = Rc::new(Kernel {
            mock_queue: Default::default(),
            syscall_log: Default::default(),
        });
        FAKE.with(|cell| {
            if cell.replace(Rc::downgrade(&rc)).strong_count() != 0 {
                panic!("New Kernel created before the previous one was dropped.");
            }
        });
        rc
    }

    // Appends a log entry to the system call queue.
    fn log_syscall(&self, syscall: crate::SyscallLogEntry) {
        let mut log = self.syscall_log.take();
        log.push(syscall);
        self.syscall_log.set(log);
    }

    // Retrieves the first mock in the mock queue, removing it from the queue.
    // Returns None if the mock queue was empty.
    fn pop_mock(&self) -> Option<crate::Mock> {
        let mut queue = self.mock_queue.take();
        let mock = queue.pop_front();
        self.mock_queue.set(queue);
        mock
    }

    /// Adds a Mock to the mock queue.
    ///
    /// # What is the mock queue?
    ///
    /// In addition to routing system calls to drivers, `Kernel` supports
    /// injecting artificial system call responses. The primary use case for
    /// this feature is to simulate errors without having to implement error
    /// simulation in each `fake::Driver`.
    ///
    /// The mock queue is a FIFO queue containing anticipated upcoming system
    /// calls. It starts empty, and as long as it is empty, mocking behavior is
    /// disabled. When the mock queue is nonemptyand a system call is made, the
    /// system call is compared with the next queue entry. If the system call
    /// matches, then the action defined by the mock queue entry is taken. If
    /// the call does not match, the call panics (to make the unit test fail).
    pub fn push_mock(&self, mock: crate::Mock) {
        let mut queue = self.mock_queue.take();
        queue.push_back(mock);
        self.mock_queue.set(queue);
    }

    /// Returns the system call log and empties it.
    pub fn take_syscall_log(&self) -> Vec<crate::SyscallLogEntry> {
        self.syscall_log.take()
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

// Returns this thread's Kernel instance. `caller` is used only to give a useful
// error message.
fn get_kernel(caller: &str) -> Rc<Kernel> {
    let clone = FAKE.with(|cell| {
        let weak = cell.replace(Weak::new());
        let clone = weak.clone();
        cell.replace(weak);
        clone
    });
    clone
        .upgrade()
        .unwrap_or_else(|| panic!("{} called after fake::Kernel was dropped", caller))
}
