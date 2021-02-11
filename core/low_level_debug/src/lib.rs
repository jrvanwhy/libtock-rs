#![no_std]

pub struct LowLevelDebug<Syscalls: libtock_platform::Syscalls> {
    _syscalls: core::marker::PhantomData<Syscalls>,
}

impl<Syscalls: libtock_platform::Syscalls> LowLevelDebug<Syscalls> {
    pub fn alert_code(alert_code: AlertCode) {
        Syscalls::command(8, 1, alert_code as u32, 0);
    }

    pub fn print1(value: u32) {
        Syscalls::command(8, 2, value, 0);
    }

    pub fn print2(value1: u32, value2: u32) {
        Syscalls::command(8, 3, value1, value2);
    }
}

pub enum AlertCode {
    Panic = 0x01,
    WrongLocation = 0x02,
}
