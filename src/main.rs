// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

use core::panic::PanicInfo;
use crate::loader::{gdt, loader_api};

use crate::vga::{cls, put_char, puts};

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

mod vga;
mod loader;
mod idt;


/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() -> ! {
    cls();
    let api = loader_api();
    println!("{}", api(3, cb as usize as u32));
    println!("{}", cb as usize);
    println!("{}", cb as usize);
    loop {}

}

extern "C" fn cb(t: u32) {
   println!("call back");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}