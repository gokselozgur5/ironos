//! ironos kernel entry, self-tests, and panic handler.

#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

use ironos_lib::allocator::FrameAllocator;
use ironos_lib::ringbuf::RingBuffer;
use ironos_lib::sv39::{PageTableEntry, Sv39VirtAddr};

pub mod sbi;
#[macro_use]
pub mod uart;

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    la sp, __stack_top

    la t0, __bss_start
    la t1, __bss_end
1:
    bgeu t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:
    tail kernel_main
    "#
);

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    kprintln!("[ironos] boot ok (riscv64 virt)");

    // Run self-tests
    let alloc_pass = test_allocator();
    let ringbuf_pass = test_ringbuf();
    let sv39_pass = test_sv39();

    kprintln!(
        "[ironos] selftest: allocator={} ringbuf={} sv39={}",
        if alloc_pass { "PASS" } else { "FAIL" },
        if ringbuf_pass { "PASS" } else { "FAIL" },
        if sv39_pass { "PASS" } else { "FAIL" }
    );

    kprintln!("[ironos] halt");
    sbi::shutdown();
}

fn test_allocator() -> bool {
    let mut alloc = FrameAllocator::<2>::new(0x8040_0000);
    let f0 = alloc.alloc_frame();
    let f1 = alloc.alloc_frame();
    match (f0, f1) {
        (Some(a0), Some(a1)) => {
            let freed = alloc.free_frame(a0);
            let double_free_rejected = !alloc.free_frame(a0);
            a0 == 0x8040_0000 && a1 == 0x8040_1000 && freed && double_free_rejected
        }
        _ => false,
    }
}

fn test_ringbuf() -> bool {
    let mut ring = RingBuffer::<4>::new();
    let p0 = ring.push(b'A').is_ok();
    let p1 = ring.push(b'B').is_ok();
    let b0 = ring.pop() == Some(b'A');
    let p2 = ring.push(b'C').is_ok();
    let p3 = ring.push(b'D').is_ok();
    let b1 = ring.pop() == Some(b'B');
    let b2 = ring.pop() == Some(b'C');
    let b3 = ring.pop() == Some(b'D');
    p0 && p1 && b0 && p2 && p3 && b1 && b2 && b3 && ring.is_empty()
}

fn test_sv39() -> bool {
    let vaddr = 0x8020_0000;
    let decoded = Sv39VirtAddr::from_vaddr(vaddr);
    let pte = PageTableEntry::new(
        vaddr,
        PageTableEntry::V | PageTableEntry::R | PageTableEntry::X,
    );
    decoded.is_aligned() && pte.is_valid() && pte.is_leaf() && pte.paddr() == vaddr
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("[ironos panic] {}", info);
    sbi::shutdown();
}
