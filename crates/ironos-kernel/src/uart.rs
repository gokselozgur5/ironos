//! NS16550A MMIO UART Driver for QEMU virt (`0x1000_0000`).

use core::fmt::{self, Write};
use core::ptr::{read_volatile, write_volatile};

/// Base physical MMIO address for UART0 on QEMU virt.
pub const UART_BASE: usize = 0x1000_0000;

#[allow(dead_code)]
const RHR: usize = 0; // Receive Holding Register (read)
const THR: usize = 0; // Transmit Holding Register (write)
const LSR: usize = 5; // Line Status Register
const LSR_TX_IDLE: u8 = 1 << 5; // Transmitter holding register empty

/// NS16550A UART device wrapper.
pub struct Uart {
    base: usize,
}

impl Uart {
    /// Creates a UART driver instance for the given base address.
    #[must_use]
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    /// Writes a single byte over UART, busy-waiting until TX is ready.
    pub fn write_byte(&self, byte: u8) {
        let lsr_ptr = (self.base + LSR) as *const u8;
        let thr_ptr = (self.base + THR) as *mut u8;

        unsafe {
            while (read_volatile(lsr_ptr) & LSR_TX_IDLE) == 0 {
                core::hint::spin_loop();
            }
            write_volatile(thr_ptr, byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// Print formatted text to the kernel UART.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::uart::_print(format_args!($($arg)*))
    };
}

/// Print formatted text followed by newline to the kernel UART.
#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::kprint!("\n")
    };
    ($($arg:tt)*) => {
        $crate::kprint!("{}\n", format_args!($($arg)*))
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut uart = Uart::new(UART_BASE);
    let _ = uart.write_fmt(args);
}
