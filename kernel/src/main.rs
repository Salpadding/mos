// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

use core::panic::PanicInfo;

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

mod vga;
mod idt;
mod asm;
mod page;
mod mem;


/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() -> ! {
    if !asm::page_enabled() {
        let stack_high = page::init_page();
        asm::page_setup(stack_high)
    } else {
        idt::init_all();
        println!("hello world");
        loop {}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{:#?}", _info);
    loop {}
}