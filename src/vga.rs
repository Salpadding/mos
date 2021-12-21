use core::fmt;
use core::fmt::Write;

const VGA_START: usize = 0xc00b8000;
const VGA_WORDS: usize = 0x4000;
const VGA_LINES: usize = 25;
const VGA_COLS: usize = 80;

static mut VGA_COL: usize = 0;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!($($arg)*));
        $crate::vga::put_char(b'\n');
    };
}

struct Writer {}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        puts(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut w = Writer {};
    w.write_fmt(args);
}

pub fn buf() -> &'static mut [u16] {
    unsafe {
        core::slice::from_raw_parts_mut(
            VGA_START as *mut _,
            VGA_WORDS,
        )
    }
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
    for c in s.as_bytes().iter() {
        put_char(*c);
    }
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
        vga[(VGA_LINES - 1) * VGA_COLS + VGA_COL] = cx;

        if VGA_COL == VGA_COLS - 1 {
            VGA_COL = 0;
            next_line();
        } else {
            VGA_COL += 1;
        }
    }
}
