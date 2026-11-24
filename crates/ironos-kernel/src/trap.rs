//! Trap handling and timer interrupt processing for S-mode.

use core::arch::global_asm;
use core::sync::atomic::AtomicU64;

/// Monotonic timer tick counter incremented by timer interrupts.
pub static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

pub(crate) const TIMER_INTERVAL: u64 = 1_000_000;
pub(crate) const SCAUSE_TIMER_INTERRUPT: u64 = (1 << 63) | 5;

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
