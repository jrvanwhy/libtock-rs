use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::convert::TryInto;
use std::rc::{Rc, Weak};

pub struct FakeSyscalls {
    drivers: Cell<HashMap<u32, std::rc::Rc<dyn Driver>>>,
    callback_queue: Cell<HashMap<(u32, u32), CallbackQueue>>,
}

impl FakeSyscalls {
    pub fn new() -> Rc<FakeSyscalls> {
        let rc = Rc::new(
            FakeSyscalls { drivers: Cell::new(HashMap::new()),
                           callback_queue: Cell::new(HashMap::new()) }
        );
        FAKE.with(|cell| {
            if cell.replace(Rc::downgrade(&rc)).strong_count() != 0 {
                panic!("New FakeSyscalls created before previous one dropped");
            }
        });
        rc
    }

    pub fn add_driver(&self, driver: std::rc::Rc<dyn Driver>) {
        let driver_num = driver.driver_num();
        let mut drivers = self.drivers.take();
        match drivers.entry(driver_num) {
            Occupied(_) =>
                panic!("FakeSyscalls: tried to add duplicate driver number {}",
                       driver_num),

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

impl libtock_platform::Syscalls for FakeSyscalls {
    unsafe fn raw_const_allow(major: usize, minor: usize, slice: *const u8, len: usize) {
        let major = major.try_into()
            .expect("Tried to call driver number {}, which is > u32::MAX");
        let minor = minor.try_into()
            .expect("Tried to call driver number {}, which is > u32::MAX");

        use std::slice::from_raw_parts;
        let this = get_fake();
        let drivers = this.drivers.take();
        let opt_driver = drivers.get(&major).cloned();
        this.drivers.set(drivers);
        if let Some(driver) = opt_driver {
            driver.const_allow(minor, from_raw_parts(slice, len));
        }
    }

    unsafe fn raw_subscribe(major: usize,
                            minor: usize,
                            callback: unsafe extern fn(usize, usize, usize, usize),
                            data: usize) {
        let major = major.try_into().expect(&format!("Major {} is too large.", major));
        let minor = minor.try_into().expect(&format!("Minor {} is too large.", minor));
        let this = get_fake();
        let mut callback_queue = this.callback_queue.take();
        let entry = callback_queue.entry((major, minor));
        let generation = match entry {
            Occupied(ref cq) => cq.get().generation + 1,
            Vacant(_) => 0,
        };
        let new_queue = CallbackQueue { fcn: callback,
                                        data,
                                        generation,
                                        queued: Vec::new() };
        match entry {
            Occupied(mut cq) => { cq.insert(new_queue); },
            Vacant(cq) => { cq.insert(new_queue); },
        }
        this.callback_queue.set(callback_queue);
        let drivers = this.drivers.take();
        let opt_driver = drivers.get(&major).cloned();
        this.drivers.set(drivers);
        if let Some(driver) = opt_driver {
            driver.subscribe(minor, Callback { driver: major, minor, generation});
        }
    }

    fn command(major: usize, minor: usize, arg1: usize, arg2: usize) {
        let major = major.try_into().expect(&format!("major {} too large", major));
        let this = get_fake();
        let drivers = this.drivers.take();
        let opt_driver = drivers.get(&major).cloned();
        this.drivers.set(drivers);
        if let Some(driver) = opt_driver {
            driver.command(minor.try_into().expect("minor too large"),
                           arg1.try_into().expect(&format!("arg1 {} too large", arg1)),
                           arg2.try_into().expect(&format!("arg2 {} too large", arg2)));
        }
    }

    fn yieldk() {
        let this = get_fake();
        let mut callback_queue = this.callback_queue.take();
        let (fcn, data, callback_args) = callback_queue
            .iter_mut()
            .filter_map(|(_, cq)| {
                cq.queued.pop().map(|args| (cq.fcn, cq.data, args))
            })
            .next()
            .expect("Process hang: yieldk called with no queued callbacks");
        this.callback_queue.set(callback_queue);
        unsafe {
            fcn(callback_args.arg1 as usize,
                callback_args.arg2 as usize,
                callback_args.arg3 as usize,
                data);
        }
    }
}

std::thread_local!(static FAKE: Cell<Weak<FakeSyscalls>> = Cell::new(Weak::new()));

fn get_fake() -> Rc<FakeSyscalls> {
    let clone = FAKE.with(|cell| {
        let weak = cell.replace(Weak::new());
        let clone = weak.clone();
        cell.replace(weak);
        clone
    });
    clone.upgrade().expect("Syscall called after FakeSyscalls dropped")
}

#[derive(Clone, Copy)]
pub struct Callback {
    driver: u32,
    minor: u32,
    generation: u64,
}

impl Callback {
    fn call(&self, arg1: u32, arg2: u32, arg3: u32) {
        let fake = get_fake();
        let mut callback_queue = fake.callback_queue.take();
        let entry = callback_queue.get_mut(&(self.driver, self.minor))
            .expect("Callback called with nonexistent queue");
        if self.generation < entry.generation {
            return;
        }
        entry.queued.push(CallbackArgs { arg1, arg2, arg3 });
        fake.callback_queue.set(callback_queue);
    }
}

pub trait Driver {
    fn driver_num(&self) -> u32;

    fn command(&self, minor: u32, arg1: u32, arg2: u32);
    fn const_allow(&self, minor: u32, buffer: &'static [u8]);
    fn subscribe(&self, minor: u32, callback: Callback);
}

struct CallbackArgs {
    arg1: u32,
    arg2: u32,
    arg3: u32,
}

struct CallbackQueue {
    fcn: unsafe extern fn(usize, usize, usize, usize),
    data: usize,
    generation: u64,
    queued: Vec<CallbackArgs>,
}

pub struct FakeConsole {
    write_buffer: Cell<&'static [u8]>,
    write_callback: Cell<Option<Callback>>,
    output: Cell<Vec<u8>>,
}

impl FakeConsole {
    pub fn new() -> Rc<FakeConsole> {
        Rc::new(FakeConsole {
            write_buffer: Cell::new(&[]),
            write_callback: Cell::new(None),
            output: Cell::new(Vec::new()),
        })
    }

    pub fn get_output(&self) -> Vec<u8> {
        let vec = self.output.take();
        let out = vec.clone();
        self.output.set(vec);
        out
    }
}

impl Driver for FakeConsole {
    fn driver_num(&self) -> u32 { 1 }

    fn command(&self, minor: u32, arg1: u32, _arg2: u32) {
        match minor {
            1 => {
                let mut vec = self.output.take();
                vec.extend_from_slice(self.write_buffer.get());
                self.output.set(vec);
                self.write_callback.get().map(|c| c.call(arg1, 0, 0));
            },
            _ => {},
        }
    }

    fn const_allow(&self, minor: u32, buffer: &'static [u8]) {
        match minor {
            1 => {
                self.write_buffer.set(buffer);
            },
            _ => {},
        }
    }

    fn subscribe(&self, minor: u32, callback: Callback) {
        match minor {
            1 => {
                self.write_callback.set(Some(callback));
            },
            _ => {},
        }
    }
}

pub fn silent_leak<T>(container: Box<T>) -> &'static mut T {
    use once_cell::sync::Lazy;
    use std::sync::atomic::AtomicPtr;
    use std::sync::Mutex;
    static LEAKS: Lazy<Mutex<Vec<AtomicPtr<u8>>>> = Lazy::new(|| Mutex::new(Vec::new()));
    let leaked = Box::leak(container);
    LEAKS.lock().unwrap_or_else(|e| e.into_inner()).push(AtomicPtr::new(leaked as *mut _ as *mut u8));
    leaked
}

#[macro_export]
macro_rules! test_component {
    [$locator:ident, $global:ident; $name:ident: $comp:ty = $init:expr] => {
        std::thread_local!{static $global: core::cell::Cell<Option<&'static $comp>>
                           = core::cell::Cell::new(None)}
        let mut $name: &'static $comp = $crate::silent_leak(std::boxed::Box::new($init));
        $global.with(|cell| cell.set(Some($name)));
        struct $locator;
        impl libtock_platform::Locator for $locator {
            type T = $comp;
            fn locate() -> &'static $comp {
                $global.with(|cell| {
                    cell.get().expect("locate called on unregistered component")
                })
            }
        }
    };
}
