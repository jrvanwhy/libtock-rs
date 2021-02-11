use libtock_platform::{RawSyscalls, YieldNoWaitReturn};

impl RawSyscalls for crate::TockSyscalls {
    unsafe fn yield1(op: u32) {
        asm!("svc 0",
             inlateout("r0") op => _, // a1
             lateout("r1") _,         // a2
             lateout("r2") _,         // a3
             lateout("r3") _,         // a4
             // r4-r8 are callee-saved.
             // r9 is platform-specific. We don't use it in libtock_runtime, so
             // it is either unused or used as a callee-saved register.
             // r10 and r11 are callee-saved.
             lateout("r12") _, // ip
             // r13 is the stack pointer and must be restored by the callee.
             lateout("r14") _, // lr
             // r15 is the program counter.
        );
    }

    unsafe fn yield2(op: u32, flag: *mut YieldNoWaitReturn) {
        asm!("svc 0",
             inlateout("r0") op => _,   // a1
             inlateout("r1") flag => _, // a2
             lateout("r2") _,           // a3
             lateout("r3") _,           // a4
             // r4-r8 are callee-saved.
             // r9 is platform-specific. We don't use it in libtock_runtime,
             // so it is either unused or used as a callee-saved register.
             // r10 and r11 are callee-saved.
             lateout("r12") _, // ip
             // r13 is the stack pointer and must be restored by the callee.
             lateout("r14") _, // lr
             // r15 is the program counter.
        );
    }

    unsafe fn syscall1<const CLASS: u32>(mut r0: u32) -> (u32, usize) {
        let r1;
        asm!("svc {}",
             const CLASS,
             inlateout("r0") r0,
             lateout("r1") r1,
             options(preserves_flags, nostack, nomem),
        );
        (r0, r1)
    }

    unsafe fn syscall2<const CLASS: u32>(mut r0: u32, mut r1: usize) -> (u32, usize) {
        asm!("svc {}",
             const CLASS,
             inlateout("r0") r0,
             inlateout("r1") r1,
             options(preserves_flags, nostack, nomem)
        );
        (r0, r1)
    }

    unsafe fn syscall4<const CLASS: u32>(
        mut r0: u32,
        mut r1: usize,
        mut r2: usize,
        mut r3: usize,
    ) -> (u32, usize, usize, usize) {
        asm!("svc {}",
             const CLASS,
             inlateout("r0") r0,
             inlateout("r1") r1,
             inlateout("r2") r2,
             inlateout("r3") r3,
             options(preserves_flags, nostack),
        );
        (r0, r1, r2, r3)
    }
}
