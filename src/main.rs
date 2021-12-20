// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

use core::panic::PanicInfo;
use core::sync::atomic;
use core::sync::atomic::Ordering;


#[no_mangle]
/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
fn _start() -> ! {
    put_char('c' as u8);
    loop {
    }
}

fn put_char(s: u8) {
    let x: *mut u16 = (0xb8000 + 80 * 2) as usize as *mut _;
    unsafe {
        let y: u16 = (s as u16) | 0x0f00;
        *x = y;
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}