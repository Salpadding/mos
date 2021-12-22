// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

use core::panic::PanicInfo;
use crate::vga::{cls, put_char, puts};

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

mod vga;
mod idt;
mod asm;
mod page;


/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() -> ! {
    if !asm::page_enabled() {
        page::init_page();
        println!("hello world1");
        asm::page_setup()
    } else {
        println!("hello world2");
        loop {}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}