// allow inline assembly
#![feature(asm)]
// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]
// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]
#![feature(unchecked_math)]

use core::panic::PanicInfo;

use crate::asm::switch_to;
use crate::mem::Pool;

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

mod asm;
mod err;
mod idt;
mod mem;
mod thread;
mod timer;
mod vga;

/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() -> ! {
    use crate::mem::page_enabled;

    if !page_enabled() {
        // setup page, page allocator, init thread pcb, jump to _start()
        crate::mem::init_page()
    } else {
        // load interrupt descriptor table
        idt::init();

        // add main thread into list, register scheduler
        crate::thread::init();

        // increase interrupt frequency
        crate::timer::init();
        // enable interrupt
        asm::sti();
        thread::new_thread(th, "new", 255);
        loop {}
    }
}

extern "C" fn th() {
    loop {
        println!("new thread is running");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{:#?}", _info);
    loop {}
}
