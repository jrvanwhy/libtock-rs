/// A mock is an anticipated upcoming system call, and the response the kernel
/// should take to the system call. Used by `fake::Kernel`'s "mock queue"
/// functionality.
#[derive(Debug)]
pub enum Mock {
    /// An upcoming `yield-wait` call. `yield-wait` has no arguments to match on
    /// or return value to set.
    YieldWait,

    /// An upcoming `yield-no-wait` call. `yield-no-wait` has a single return
    /// value: whether to indicate a callback ran.
    YieldNoWait { callback_ran: bool },

    /// An upcoming `command` call. This allows us to mock calls with unknown
    /// return variants, to confirm drivers are robust against unexpected return
    /// types.
    Command {
        driver_id: u32,
        command: u32,
        arg1: u32,
        arg2: u32,
        output: libtock_platform::CommandReturn,
    },

    /// An upcoming `allow-read-write` call.
    AllowRw {
        driver_id: u32,
        buffer_id: u32,
        // If an Err is passed, the buffer passed to allow-read-write is returned.
        output: Result<&'static mut [u8], libtock_platform::ErrorCode>,
    },
}

impl Mock {
    // Panics with a message describing that the named system call was called
    // instead of the mocked system call.
    pub(crate) fn panic_wrong_call(&self, called: &str) -> ! {
        panic!(
            "Mocked system call {:?}, but {} was called instead.",
            self, called
        );
    }
}
