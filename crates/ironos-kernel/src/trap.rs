//! Trap handling and timer interrupt processing for S-mode.

use core::arch::global_asm;
use core::sync::atomic::AtomicU64;

/// Monotonic timer tick counter incremented by timer interrupts.
pub static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

pub(crate) const TIMER_INTERVAL: u64 = 1_000_000;
pub(crate) const SCAUSE_TIMER_INTERRUPT: u64 = (1 << 63) | 5;

/// Returns current monotonic timer tick count.
#[must_use]
pub fn ticks() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

global_asm!(
    r#"
    .section .text.trap
    .globl trap_vector
    .align 4
trap_vector:
    addi sp, sp, -128
    sd ra, 0(sp)
    sd t0, 8(sp)
    sd t1, 16(sp)
    sd t2, 24(sp)
    sd t3, 32(sp)
    sd t4, 40(sp)
    sd t5, 48(sp)
    sd t6, 56(sp)
    sd a0, 64(sp)
    sd a1, 72(sp)
    sd a2, 80(sp)
    sd a3, 88(sp)
    sd a4, 96(sp)
    sd a5, 104(sp)
    sd a6, 112(sp)
    sd a7, 120(sp)

    call rust_trap_handler

    ld ra, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld t2, 24(sp)
    ld t3, 32(sp)
    ld t4, 40(sp)
    ld t5, 48(sp)
    ld t6, 56(sp)
    ld a0, 64(sp)
    ld a1, 72(sp)
    ld a2, 80(sp)
    ld a3, 88(sp)
    ld a4, 96(sp)
    ld a5, 104(sp)
    ld a6, 112(sp)
    ld a7, 120(sp)
    addi sp, sp, 128
    sret
    "#
);

use core::arch::asm;
use core::sync::atomic::Ordering;

use crate::sbi;

extern "C" {
    fn trap_vector();
}

/// Initializes S-mode trap vector, enables STIE in sie, and arms the first timer interrupt.
pub fn init() {
    unsafe {
        let vec = trap_vector as *const () as usize;
        asm!("csrw stvec, {0}", in(reg) vec);
        // Cleanly initialize sie register with STIE (bit 5) enabled
        asm!("csrw sie, {0}", in(reg) 1usize << 5);
        // Enable SIE (bit 1) in sstatus register
        asm!("csrs sstatus, {0}", in(reg) 1usize << 1);
    }
    sbi::set_timer(sbi::rdtime() + TIMER_INTERVAL);
}

#[no_mangle]
extern "C" fn rust_trap_handler() {
    let scause: u64;
    unsafe {
        asm!("csrr {0}, scause", out(reg) scause);
    }

    if scause == SCAUSE_TIMER_INTERRUPT {
        TICK_COUNT.fetch_add(1, Ordering::Relaxed);
        sbi::set_timer(sbi::rdtime() + TIMER_INTERVAL);
    } else {
        let sepc: u64;
        let stval: u64;
        unsafe {
            asm!("csrr {0}, sepc", out(reg) sepc);
            asm!("csrr {0}, stval", out(reg) stval);
        }
        crate::kprintln!(
            "[ironos trap] unhandled scause={:#x} sepc={:#x} stval={:#x}",
            scause,
            sepc,
            stval
        );
        sbi::shutdown();
    }
}
