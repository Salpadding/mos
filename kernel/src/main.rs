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

use crate::mem::Pool;

macro_rules! cst {
    ($p: expr) => {
        unsafe { &mut *($p as *mut _) }
    };
}
macro_rules! bk {
    () => {
        unsafe {
            asm!("2:", "jmp 2b", "nop", "nop");
        }
    };
}

macro_rules! bp {
    () => {
        {
            let a: u32;
            unsafe {
                asm!("mov {}, ebp", out(reg) a);
            }
            a
        }
    };
}

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

mod asm;
mod err;
mod init;
mod int;
mod mem;
mod thread;
mod timer;
mod vga;

/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() {
    use crate::mem::page_enabled;

    if !*page_enabled() {
        crate::vga::init_com1();
        asm::init();
        // setup page, page allocator, init thread pcb, jump to _start()
        *page_enabled() = true;
        crate::thread::tss::init();
        crate::mem::init_page();
    } else {
        // load interrupt descriptor table
        int::init();

        // add main thread into list, register scheduler
        crate::thread::init();
        crate::init::init_locks();

        // increase interrupt frequency
        crate::timer::init();
        crate::thread::user::create(th_print_d, 1, "th0", 1);

        // enable interrupt
        asm::sti();

        bk!();
    }
}

extern "C" fn th_print_d(d: usize) {
    loop {
        c_println!("{:02X} ", d);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    c_println!("{:#?}", _info);
    loop {}
}
