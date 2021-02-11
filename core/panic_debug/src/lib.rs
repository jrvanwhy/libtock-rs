#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use libtock_low_level_debug::{AlertCode, LowLevelDebug};
    LowLevelDebug::<libtock_runtime::TockSyscalls>::alert_code(AlertCode::Panic);

    // TODO: Add code that formats and prints debug information to the console.

    // TODO: Replace with exit when exit is implemented.
    loop {
        use libtock_platform::Syscalls;
        libtock_runtime::TockSyscalls::yield_wait();
    }
}
