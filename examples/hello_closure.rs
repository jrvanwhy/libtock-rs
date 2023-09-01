#![no_main]
#![no_std]

use core::cell::Cell;
use libtock::platform::{AllowRo, AllowRw, DefaultConfig, ErrorCode, share, Subscribe, Syscalls};
use libtock::runtime::{set_main, stack_size, TockSyscalls};

set_main! {main}
stack_size! {0x100}

mod alarm {
    use super::*;
    pub type WakeupHandles<'share> = Subscribe<'share, TockSyscalls, 0, 0>;
    pub fn wakeup<'share>(milliseconds: u32, done: &'share Cell<bool>, handle: share::Handle<WakeupHandles<'share>>) -> Result<(), ErrorCode> {
        let frequency = TockSyscalls::command(0, 1, 0, 0).to_result::<u32, _>()?;
        let ticks = milliseconds * frequency / 1000;
        TockSyscalls::subscribe::<_, _, DefaultConfig, 0, 0>(handle, done)?;
        TockSyscalls::command(0, 5, ticks, 0).to_result::<u32, _>()?;
        Ok(())
    }
}

mod console {
    use super::*;
    pub type ReadHandles<'share> = (AllowRw<'share, TockSyscalls, 1, 1>, Subscribe<'share, TockSyscalls, 1, 2>);
    pub fn read<'share>(buffer: &'share mut [u8], bytes: &'share Cell<Option<(u32, u32)>>, handle: share::Handle<ReadHandles<'share>>) -> Result<(), ErrorCode> {
        let (allow_handle, subscribe_handle) = handle.split();
        TockSyscalls::subscribe::<_, _, DefaultConfig, 1, 2>(subscribe_handle, bytes)?;
        let len = buffer.len() as u32;
        TockSyscalls::allow_rw::<DefaultConfig, 1, 1>(allow_handle, buffer)?;
        TockSyscalls::command(1, 2, len, 0).to_result()
    }
    pub type WriteHandles<'share> = (AllowRo<'share, TockSyscalls, 1, 1>, Subscribe<'share, TockSyscalls, 1, 1>);
    pub fn write<'share>(message: &'share [u8], done: &'share Cell<bool>, handle: share::Handle<WriteHandles<'share>>) -> Result<(), ErrorCode> {
        let (allow_handle, subscribe_handle) = handle.split();
        TockSyscalls::allow_ro::<DefaultConfig, 1, 1>(allow_handle, message)?;
        TockSyscalls::subscribe::<_, _, DefaultConfig, 1, 1>(subscribe_handle, done)?;
        TockSyscalls::command(1, 1, message.len() as u32, 0).to_result()
    }
}

fn main() -> Result<(), ErrorCode> {
    let mut buffer = *b"Hello,                   ";
    let bytes = Cell::new(None);
    let timeout = Cell::new(false);
    let receive_len = share::scope(|handles: share::Handle<(alarm::WakeupHandles, console::ReadHandles)>| {
        let (alarm_handles, console_handles) = handles.split();
        console::read(&mut buffer[7..23], &bytes, console_handles)?;
        alarm::wakeup(1000, &timeout, alarm_handles)?;
        loop {
            TockSyscalls::yield_wait();
            if let Some((_, len)) = bytes.get() {
                break Ok(len as usize);
            }
            if timeout.get() {
                break Ok(0);
            }
        }
    })?;
    // Console *shouldn't* return a receive_len greater than 16, but if it bugs
    // out it could. This check is needed to prevent subsequent indexing
    // operations from panicing, which allows the compiler to optimize the
    // panics away.
    if receive_len > 16 {
        return Err(ErrorCode::Fail);
    }
    let name_len = match receive_len {
        0 => {
            let world: &mut [u8; 5] = (&mut buffer[7..12]).try_into().unwrap();
            *world = *b"World";
            world.len()
        },
        len => len,
    };
    buffer[7 + name_len] = b'!';
    buffer[8 + name_len] = b'\n';
    let msg_len = 9 + name_len;
    let done = Cell::new(false);
    let to_print = &buffer[..msg_len];
    share::scope(|handle| {
        let _ = console::write(to_print, &done, handle);
        loop {
            TockSyscalls::yield_wait();
        }
    });
    Ok(())
}
