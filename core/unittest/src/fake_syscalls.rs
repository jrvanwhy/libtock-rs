use crate::FakeDriver;
use libtock_platform::{OneArgMemop, RawSyscalls, return_type, YieldType, ZeroArgMemop};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};
use std::thread_local;

/// A fake implementation of the Tock system calls. Provides
/// `libtock_platform::Syscalls` by implementing
/// `libtock_platform::RawSyscalls`. Allows `FakeDriver`s to be attached, and
/// routes system calls to the correct fake driver.
///
/// Note that there can only be one `FakeSyscalls` instance per thread, as a
/// thread-local variable is used to implement `libtock_platform::RawSyscalls`.
/// As such, test code is given a `Rc<FakeSyscalls>` rather than a
/// `FakeSyscalls` instance directly.
pub struct FakeSyscalls {
    // Map from driver_id to FakeDriver instance.
    drivers: Cell<HashMap<u32, Rc<dyn FakeDriver>>>,

    // The callback queue and generation values. Each time `subscribe` is
    // called, the corresponding generation value is incremented. Callbacks are
    // only called if they haven't been replaced by a newer callback, which is
    // detected using the generation value.
    callback_queue: Cell<VecDeque<QueuedCallback>>,

    // Map from (driver_id, subscribe_id) to the current callback function.
    // Safety invariant: every CurrentCallback in callbacks is still active
    // (i.e. has not been replaced by a newer subscribe call).
    callbacks: Cell<HashMap<(u32, u32), CurrentCallback>>,
}

impl FakeSyscalls {
    /// Creates a `FakeSyscalls` for this thread and returns a reference to it.
    /// If there is already a `FakeSyscalls` for this thread, `new` panics.
    pub fn new() -> Rc<FakeSyscalls> {
        let rc = Rc::new(
            FakeSyscalls { drivers: Cell::new(HashMap::new()),
                           callback_queue: Cell::new(VecDeque::new()),
                           callbacks: Cell::new(HashMap::new()) }
        );
        FAKE.with(|cell| {
            if cell.replace(Rc::downgrade(&rc)).strong_count() != 0 {
                panic!("New FakeSyscalls created before the previous one was dropped.");
            }
        });
        rc
    }

    /// Adds a driver to this FakeSyscalls instance. System calls with a
    /// `driver_id` that matches this driver will be routed to this driver
    /// instance. Note that you cannot add two drivers with the same
    /// `driver_id`; `add_driver` will panic if you try to do so.
    pub fn add_driver(&self, driver: Rc<dyn FakeDriver>) {
        use std::collections::hash_map::Entry::{Occupied, Vacant};
        let driver_id = driver.driver_id();
        let mut drivers = self.drivers.take();
        match drivers.entry(driver_id) {
            Occupied(_) =>
                panic!("FakeSyscalls: tried to add duplicate driver with ID {}",
                       driver_id),
            Vacant(entry) => { entry.insert(driver); },
        }
        self.drivers.set(drivers);
    }
}

impl Drop for FakeSyscalls {
    fn drop(&mut self) {
        FAKE.with(|cell| cell.replace(Weak::new()));
    }
}

impl RawSyscalls for FakeSyscalls {
    fn raw_yield(r0_in: YieldType) -> usize {
        let this = get_fake("yield");

        // Find the next callback, if one is available.
        let mut callback_queue = this.callback_queue.take();
        let callbacks = this.callbacks.take();
        let callback: Option<(&CurrentCallback, QueuedCallback)> = loop {
            let queued_callback = match callback_queue.pop_front() {
                None => break None,
                Some(queued_callback) => queued_callback,
            };

            let current_callback =
                callbacks.get(&(queued_callback.driver_id, queued_callback.subscribe_id))
                         .expect("Queued callback with no current callback");

            // Skip callbacks that have been replaced by a newer subscribe call.
            if queued_callback.generation != current_callback.generation {
                continue;
            }

            break Some((current_callback, queued_callback));
        };
        this.callback_queue.set(callback_queue);

        if let Some((ref current_callback, ref queued_callback)) = callback {
            unsafe { current_callback.call(queued_callback); }
        }
        let callback_executed = callback.is_some();
        this.callbacks.set(callbacks);

        match r0_in {
            YieldType::Wait => {
                if !callback_executed {
                    panic!("App deadlock: yield_wait called with no queued callback");
                }
                return_type::SUCCESS
            },
            YieldType::NoWait => {
                match callback_executed {
                    true => return_type::SUCCESS,
                    false => return_type::FAILURE,
                }
            }
            _ => panic!("Unexpected yield type {:?}", r0_in),
        }.into()
    }

    unsafe fn four_arg_syscall(
        r0: usize,
        r1: usize,
        r2: usize,
        r3: usize,
        class: u8,
    ) -> (usize, usize, usize, usize) {
        unimplemented!("TODO");
    }

    fn zero_arg_memop(r0_in: ZeroArgMemop) -> (usize, usize) {
        unimplemented!("TODO");
    }

    fn one_arg_memop(r0_in: OneArgMemop, r1: usize) -> (usize, usize) {
        unimplemented!("TODO");
    }
}

// -----------------------------------------------------------------------------
// Implementation details below.
// -----------------------------------------------------------------------------

// A handle to this thread's FakeSyscalls instance. Used by the implementation
// of RawSyscalls on FakeSyscalls. This is a weak reference so that when the
// unit test is done with FakeSyscalls, the following cleanup can happen:
//   1. The test drops its Rc<FakeSyscalls>
//   2. The strong count drops to 0 so the FakeSyscalls is dropped.
//   3. FakeSyscalls' Drop implementation clears out FAKE.
//   4. The backing storage holding the FakeSyscalls is deallocated (no weak
//      references left).
thread_local!(static FAKE: Cell<Weak<FakeSyscalls>> = Cell::new(Weak::new()));

// Returns this thread's FakeSyscalls instance. `caller` is used only to give a
// useful error message.
pub(crate) fn get_fake(caller: &str) -> Rc<FakeSyscalls> {
    let clone = FAKE.with(|cell| {
        let weak = cell.replace(Weak::new());
        let clone = weak.clone();
        cell.replace(weak);
        clone
    });
    clone.upgrade().expect(&format!("{} called after FakeSyscalls was dropped", caller))
}

// Represents the current callback shared via subscribe for a particular
// (driver_id, subscribe_id) combination.
struct CurrentCallback {
    // Used to skip queued callbacks that have been replaced by newer callbacks.
    generation: u64,

    userdata: usize,
    fn_ptr: unsafe extern fn(u32, u32, u32, usize),
}

impl CurrentCallback {
    /// # Safety
    /// userdata and fn_ptr must have been provided together by a valid
    /// subscribe call to RawSyscalls::four_arg_syscall.
    pub unsafe fn new(generation: u64, userdata: usize, fn_ptr: unsafe extern fn(u32, u32, u32, usize)) -> Self {
        CurrentCallback { generation, userdata, fn_ptr }
    }

    /// # Safety
    /// This callback must still be valid (i.e. not have been replaced by a
    /// more recent call to subscribe).
    pub unsafe fn call(&self, queued_callback: &QueuedCallback) {
        (self.fn_ptr)(queued_callback.argument0,
                      queued_callback.argument1,
                      queued_callback.argument2,
                      self.userdata)
    }
}

struct QueuedCallback {
    driver_id: u32,
    subscribe_id: u32,
    generation: u64,
    argument0: u32,
    argument1: u32,
    argument2: u32,
}
