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
    addi sp, sp, -64
    sd ra, 0(sp)
    sd t0, 8(sp)
    sd t1, 16(sp)
    sd a0, 24(sp)
    sd a1, 32(sp)
    sd a7, 40(sp)

    call rust_trap_handler

    ld ra, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld a0, 24(sp)
    ld a1, 32(sp)
    ld a7, 40(sp)
    addi sp, sp, 64
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
        // Enable STIE (bit 5) in sie register
        asm!("csrs sie, {0}", in(reg) 1usize << 5);
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
    }
}
