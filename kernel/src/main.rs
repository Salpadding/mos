
// allow inline assembly
#![feature(asm)]
// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

use core::panic::PanicInfo;
use crate::mem::Pool;

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}



mod vga;
mod idt;
mod asm;
mod mem;
mod err;
mod thread;
mod timer;


/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() -> ! {
    use crate::mem::page_enabled;
    if !page_enabled() {
        crate::mem::init_page()
    } else {
        idt::init();
        crate::thread::init();
        crate::timer::init();
        println!("hello world");
        loop {}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("unexpected panic");
    println!("{:#?}", _info);
    loop {}
}