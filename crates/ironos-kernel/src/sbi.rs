//! OpenSBI S-mode Supervisor Binary Interface wrapper.

use core::arch::asm;

/// SBI System Reset Extension ID (SRST).
pub const SBI_EXT_SRST: usize = 0x53525354;
/// SBI SRST System Reset Function ID.
pub const SBI_SRST_RESET: usize = 0;

/// Reset types for SRST.
pub const SBI_SRST_RESET_TYPE_SHUTDOWN: usize = 0;
/// Reset reasons for SRST.
pub const SBI_SRST_RESET_REASON_NONE: usize = 0;

/// Legacy SBI Shutdown Extension ID.
pub const SBI_EXT_LEGACY_SHUTDOWN: usize = 0x08;

/// Generic SBI `ecall` with 6 arguments.
#[inline(always)]
pub fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    let mut err: usize;
    let mut val: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => err,
            inlateout("a1") arg1 => val,
            in("a2") arg2,
            options(nostack)
        );
    }
    (err, val)
}

/// Shuts down the machine via OpenSBI SRST extension, falling back to legacy SBI shutdown.
pub fn shutdown() -> ! {
    // Attempt standard SBI SRST shutdown
    let _ = sbi_call(
        SBI_EXT_SRST,
        SBI_SRST_RESET,
        SBI_SRST_RESET_TYPE_SHUTDOWN,
        SBI_SRST_RESET_REASON_NONE,
        0,
    );

    // Fallback: Legacy SBI 0.1 shutdown
    unsafe {
        asm!(
            "ecall",
            in("a7") SBI_EXT_LEGACY_SHUTDOWN,
            in("a0") 0,
            options(noreturn)
        );
    }
}
