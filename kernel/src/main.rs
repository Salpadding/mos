// allow inline assembly
#![feature(asm)]
// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]
// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]
#![feature(unchecked_math)]

macro_rules! cst {
    ($p: expr) => {
       unsafe { &mut *($p as *mut _) } 
    };
}
macro_rules! bk {
    () => {
        unsafe { asm!("2:", "jmp 2b", "nop", "nop");}
    };
}

use core::panic::PanicInfo;

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

static mut I: u64 = 0;

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

        println!("thread init success");

        // increase interrupt frequency
        crate::timer::init();
        println!("timer init success");
        // enable interrupt
        asm::sti();
        println!("sti success");
        thread::new_thread(th, "new", 1);
        // println!("thread created");
        loop{}
    }
}

extern "C" fn th() {
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{:#?}", _info);
    loop {}
}
