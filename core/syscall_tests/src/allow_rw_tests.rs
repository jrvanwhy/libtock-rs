//! Unit tests for the `allow-read-write` system call.

use libtock_platform::Syscalls;
use libtock_unittest::{fake};

/// The first call to allow with a particular buffer ID returns a 0 address.
/// This test exercises that code path.

#[test]
fn allow_rw_static_first() {
    let _kernel = fake::Kernel::new();
    let buffer = Box::leak(Box::new([0]));
    //let buffer_raw = buffer as *mut _;
    let _returned_buffer = fake::Kernel::allow_rw_static(1, 2, buffer).expect("allow_rw_static Err");
}
