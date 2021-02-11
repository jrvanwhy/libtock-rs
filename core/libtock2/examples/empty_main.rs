#![no_main]
#![no_std]

libtock2::runtime::set_main! {main}
libtock2::runtime::stack_size! {0x800}

fn main() -> ! {
    use libtock2::platform::Syscalls;
    // TODO: Make this an exit
    loop {
        libtock2::runtime::TockSyscalls::yield_wait();
    }
}
