#![no_std]

pub struct Console<Syscalls: libtock_platform::RawSyscalls> {
    _syscalls: core::marker::PhantomData<Syscalls>,
}

impl<Syscalls: libtock_platform::RawSyscalls> Console<Syscalls> {
    pub fn new() -> Self {
        Console {
            _syscalls: core::marker::PhantomData,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        use libtock_platform::Syscalls;
        let done = core::cell::Cell::new(false);
        unsafe {
            Syscalls::four_arg_syscall::<1>(1, 1, callback as usize, &done as *const _ as usize);
            Syscalls::four_arg_syscall::<4>(1, 1, s.as_ptr() as usize, s.len());
        }
        // TODO: Error handling
        Syscalls::command(1, 1, s.len() as u32, 0);
        while !done.get() {
            Syscalls::yield_wait();
        }
        unsafe {
            Syscalls::four_arg_syscall::<4>(1, 1, 0, 0);
            Syscalls::four_arg_syscall::<1>(1, 1, 0, 0);
        }
    }
}

unsafe extern "C" fn callback(_arg1: u32, _arg2: u32, _arg3: u32, done: usize) {
    (*(done as *const core::cell::Cell<bool>)).set(true);
}
