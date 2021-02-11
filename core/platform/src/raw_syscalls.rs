// TODO: Implement `libtock_unittest`, which is referenced in the comment on
// `RawSyscalls`.

/// `RawSyscalls` allows a fake Tock kernel to be injected into components for
/// unit testing. It is implemented by `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel`. **Components should not use `RawSyscalls`
/// directly; instead, use the `Syscalls` trait, which provides higher-level
/// interfaces to the system calls.**

// RawSyscalls is designed to minimize the amount of handwritten assembly code
// needed without generating unnecessary instructions. This comment describes
// the thought process that led to the choice of methods for RawSyscalls.
//
// First: The decision of where to use u32 and usize can be a bit tricky. The
// Tock syscall ABI is currently only specified for 32-bit systems, so on real
// Tock systems both types match the size of a register, but the unit test
// environment can be either 32 bit or 64 bit. This interface uses usize for
// values that can contain pointers, so that pointers are not truncated in the
// unit test environment. To keep types as consistent as possible, it uses u32
// for untyped values that cannot be pointers.
//
// Theoretically, we could use a single raw system call:
//
//   unsafe fn syscall<const CLASS>(usize, usize, usize, usize) -> (usize, usize, usize, usize);
//
// However, this has a major inefficiency. The single raw system call would need
// to clobber every register that any system call can clobber. Yield has a far
// longer clobber list than most system calls, so this would be inefficient for
// the majority of system calls. As a result, we can split yield out into its
// own function, giving the following API:
//
//   unsafe fn yield(usize, usize, usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall<const CLASS>(usize, usize, usize, usize) -> (usize, usize, usize, usize);
//
// There is one significant inefficiency remaining. Many system calls, such as
// memop's "get RAM start address" operation, do not need to set all four
// arguments. The compiler cannot optimize away this inefficiency, so to remove
// it we need to split the system calls up based on the number of arguments they
// take:
//
//   unsafe fn yield0() -> (usize, usize, usize, usize);
//   unsafe fn yield1(usize) -> (usize, usize, usize, usize);
//   unsafe fn yield2(usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn yield3(usize, usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn yield4(usize, usize, usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall0<const CLASS>() -> (usize, usize, usize, usize);
//   unsafe fn syscall1<const CLASS>(usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall2<const CLASS>(usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall3<const CLASS>(usize, usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall4<const CLASS>(usize, usize, usize, usize) -> (usize, usize, usize, usize);
//
// However, not all of these are used! If we remove the system calls that are
// unused, we are left with the following:
//
//   unsafe fn yield1(usize) -> (usize, usize, usize, usize);
//   unsafe fn yield2(usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall1<const CLASS>(usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall2<const CLASS>(usize, usize) -> (usize, usize, usize, usize);
//   unsafe fn syscall4<const CLASS>(usize, usize, usize, usize) -> (usize, usize, usize, usize);
//
// Last, the system call return value format always returns a return variant in
// r0, which is a 32-bit value. Therefore we can change the first return value
// to a u32:
//
//   unsafe fn yield1(usize) -> (u32, usize, usize, usize);
//   unsafe fn yield2(usize, usize) -> (u32, usize, usize, usize);
//   unsafe fn syscall1<const CLASS>(usize) -> (u32, usize, usize, usize);
//   unsafe fn syscall2<const CLASS>(usize, usize) -> (u32, usize, usize, usize);
//   unsafe fn syscall4<const CLASS>(usize, usize, usize, usize) -> (u32, usize, usize, usize);
//
// These system calls are refined further individually, which is documented on
// a per-function basis.
//
// Convention: This file uses the same register naming conventions as the Tock
// 2.0 syscall TRD. Registers r0-r4 correspond to ARM registers r0-r4 and RISC-V
// registers a0-a4.
pub trait RawSyscalls {
    // yield1 can only be used to call 1 yield operation, `yield-wait`, which
    // does not have a return value. Therefore we omit a return value from
    // yield1.
    //
    // yield1 should:
    //     1. Call syscall class 0
    //     2. Pass the provided operation in register r0 as an inlateout
    //        register.
    //     3. Mark all caller-saved registers as lateout clobbers.
    //     4. NOT provide any of the following options:
    //            pure             (yield has side effects)
    //            nomem            (a callback can read + write globals)
    //            readonly         (a callback can write globals)
    //            preserves_flags  (a callback can change flags)
    //            noreturn         (yield is expected to return)
    //            nostack          (a callback needs the stack)
    /// `yield1` should only be called by `libtock_platform`.
    /// # Safety
    /// yield1 may only be used for yield operations that do not return a value.
    /// It is exactly as safe as the underlying system call.
    unsafe fn yield1(op: u32);

    // yield2 can only be used to call 1 yield operation: `yield-no-wait`.
    // `yield-no-wait` does not return any values, so we omit return arguments.
    // We pass a `*mut YieldNoWaitReturn` instead of a `usize` to provide more
    // type safety.
    //
    // yield2 should:
    //     1. Call syscall class 0
    //     2. Pass op in register r0 as inlateout.
    //     3. Pass the flag pointer in register r1 as inlateout.
    //     4. Mark all caller-saved registers as lateout clobbers.
    //     5. NOT provide any of the following options:
    //            pure             (yield has side effects)
    //            nomem            (a callback can read + write globals)
    //            readonly         (a callback can write globals)
    //            preserves_flags  (a callback can change flags)
    //            noreturn         (yield is expected to return)
    //            nostack          (a callback needs the stack)
    /// `yield2` should only be called by `libtock_platform`.
    /// # Safety
    /// yield2 may only be used for the `yield-no-wait` system call. `flag` must
    /// be valid to write to, but not necessarily read from. `yield2` will set
    /// `flag` before it returns.
    unsafe fn yield2(op: u32, flag: *mut YieldNoWaitReturn);

    // syscall1 is only used to invoke memop operations. 1-argument memop calls
    // always use r0 to specify an operation, which is a 32-bit value. Because
    // there are no memop commands that set r2 or r3, raw_syscall1 only needs to
    // return r0 and r1.
    //
    // Memop commands may panic in the unit test environment, as not all memop
    // calls can be sensibly implemented in that environment.
    //
    // syscall1 should:
    //     1. Call the syscall class specified by CLASS.
    //     2. Specify r0 as an inlateout register, passing r0 in and out.
    //     3. Specify r1 as a lateout register and return its value.
    //     4. Does not mark any registers as clobbered.
    //     5. Has all of the following options:
    //            preserves_flags
    //            nostack
    //            nomem            (it is okay for the compiler to cache globals
    //                              across memop calls)
    //     6. Does NOT have any of the following options:
    //            pure      (two invocations of the same memop can return
    //                       different values)
    //            readonly  (incompatible with nomem)
    //            noreturn
    /// `syscall1` should only be called by `libtock_platform`.
    /// # Safety
    /// This directly makes a system call. It can only be used for core kernel
    /// system calls that accept 1 argument and only overwrite r0 and r1 on
    /// return. It is unsafe any time the underlying system call is unsafe.
    unsafe fn syscall1<const CLASS: u32>(r0: u32) -> (u32, usize);

    // syscall2 is used to invoke memop operations that take an argument as well
    // as exit. For both memop and exit, the value in r0 is a 32-bit identifier.
    // Memop does not currently use more than 2 registers for its return value,
    // and exit does not return, so syscall2 only returns 2 values.
    //
    // syscall2  should:
    //     1. Call the syscall class specified by CLASS.
    //     2. Specify r0 as an inlateout register, passing r0 in and out.
    //     3. Specify r1 as an inlateout register, passing r1 in and out.
    //     4. Not mark any registers as clobbered.
    //     5. Have all of the following options:
    //            preserves_flags
    //            nostack
    //            nomem            (the compiler can cache globals across memop
    //                              calls)
    //     6. Does NOT have any of the following options:
    //            pure      Two invocations of sbrk can return different values
    //            readonly  Incompatible with nomem
    //            noreturn
    /// `syscall2` should only be called by `libtock_platform`.
    /// # Safety
    /// `syscall2` directly makes a system call. It can only be used for core
    /// kernel system calls that accept 2 arguments and only overwrite r0 and r1
    /// on return. It is unsafe any time the underlying system call is unsafe.
    unsafe fn syscall2<const CLASS: u32>(r0: u32, r1: usize) -> (u32, usize);

    // syscall4 is used to invoke the subscribe, command, read-write allow, and
    // read-only allow system calls. For all those system calls, the first
    // argument is a driver ID, which is a 32-bit value.
    //
    // syscall4 should:
    //     1. Call the syscall class specified by CLASS.
    //     2. Pass r0-r3 in the corresponding registers as inlateout registers.
    //     3. Returns r0-r3 in order.
    //     4. Not mark any registers as clobbered.
    //     5. Have all of the following options:
    //            preserves_flags  (these system calls do not touch flags)
    //            nostack          (these system calls do not touch the stack)
    //     6. NOT have any of the following options:
    //            pure      (these system calls have side effects)
    //            nomem     (the compiler needs to write to globals before allow)
    //            readonly  (rw allow can modify memory)
    //            noreturn  (all these system calls are expected to return)
    //
    // For subscribe(), the callback pointer should be either 0 (for the null
    // callback) or an `unsafe extern fn(u32, u32, u32, usize)`.
    /// `syscall4` should only be called by `libtock_platform`.
    ///
    /// # Safety
    /// `syscall4` must NOT be used to invoke yield. Otherwise, it has the same
    /// safety invariants as the underlying system call, which varies depending
    /// on the system call class.
    unsafe fn syscall4<const CLASS: u32>(
        r0: u32,
        r1: usize,
        r2: usize,
        r3: usize,
    ) -> (u32, usize, usize, usize);
}

// Return flag for yield-no-wait. We cannot safely pass a `*mut bool` to the
// kernel, because the representation of `bool` in Rust is undefined (although
// it is likely false == 0, true == 1, based on `bool`'s conversions). Passing
// a `*mut YieldNoWaitReturn` rather than a `*mut u8` allows the compiler to
// assume the kernel will never write a value other than 0 or 1 into the
// pointee. Assuming the likely representation of `bool`, this makes the
// conversion into `bool` free.
/// `YieldNoWaitReturn` should only be used by `libtock_platform`.
#[derive(PartialEq)]
#[repr(u8)]
pub enum YieldNoWaitReturn {
    NoCallback = 0,
    Callback = 1,
}
