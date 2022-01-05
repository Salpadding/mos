use crate::asm::{in_b, out_b};
use core::fmt;
use core::fmt::Write;

const VGA_START: usize = 0xb8000;
const VGA_LINES: usize = 25;
const VGA_COLS: usize = 80;
const VGA_WORDS: usize = VGA_COLS * VGA_LINES;

pub static mut VGA_COL: usize = 0;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { 
        {
            if unsafe { $crate::vga::VGA_LOCK_REF == 0 } {
                $crate::vga::_print(format_args!($($arg)*), false); 
            } else {
                let lock: &'static mut $crate::thread::sync::Lock = cst!($crate::vga::VGA_LOCK_REF);
                let mut gd = lock.lock();
                $crate::vga::_print(format_args!($($arg)*), false); 
            }
        }
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => { 
        {
            if unsafe { $crate::vga::VGA_LOCK_REF == 0 } {
                $crate::vga::_print(format_args!($($arg)*), true); 
            } else {
                let lock: &'static mut $crate::thread::sync::Lock = cst!($crate::vga::VGA_LOCK_REF);
                let mut gd = lock.lock();
                $crate::vga::_print(format_args!($($arg)*), true); 
            }
        }
    };
}

struct Writer {}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        puts(s);
        Ok(())
    }
}

pub fn _print_unsafe(args: fmt::Arguments, n: bool) {
    let mut w = Writer {};
    w.write_fmt(args).unwrap();

    if n {
        next_line();
    }
}

pub fn _print(args: fmt::Arguments, n: bool) {
    _print_unsafe(args, n);
}

pub fn buf() -> &'static mut [u16] {
    unsafe { core::slice::from_raw_parts_mut(VGA_START as *mut _, VGA_WORDS) }
}

pub fn cls() {
    let vga = buf();
    for i in 0..VGA_WORDS {
        vga[i] = 0x0f20;
    }
}

pub fn next_line() {
    let vga = buf();
    for i in 0..VGA_LINES - 1 {
        for j in 0..VGA_COLS {
            vga[i * VGA_COLS + j] = vga[(i + 1) * VGA_COLS + j];
        }
    }

    for i in 0..VGA_COLS {
        vga[(VGA_LINES - 1) * VGA_COLS + i] = 0x0f20;
    }

    unsafe { VGA_COL = 0 };
}

#[no_mangle]
pub fn puts(s: &str) {
    let bytes = s.as_bytes();
    let len = s.len();
    for i in 0..len {
        put_char(bytes[i]);
    }
}

pub static mut VGA_LOCK: [u8; 256] = [0u8; 256];
pub static mut VGA_LOCK_REF: usize = 0;

const PORT: u16 = 0x3f8;

pub fn init_com1() {
    out_b(PORT + 1, 0x00); // Disable all interrupts
    out_b(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
    out_b(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    out_b(PORT + 1, 0x00); //                  (hi byte)
    out_b(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
    out_b(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    out_b(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
    out_b(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
    out_b(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)
    assert!(in_b(PORT) == 0xAE);
    out_b(PORT + 4, 0x0f);
}

#[no_mangle]
pub fn put_char(c: u8) {
    let vga = buf();

    if c == b'\n' {
        next_line();
        return;
    }

    let cx: u16 = (c as u16) | 0x0f00;
    unsafe {
        let i = (VGA_LINES - 1) * VGA_COLS + VGA_COL;
        vga[i] = cx;

        if VGA_COL == VGA_COLS - 1 {
            VGA_COL = 0;
            next_line();
        } else {
            VGA_COL += 1;
        }
    }
}
