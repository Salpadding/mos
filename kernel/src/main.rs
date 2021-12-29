// allow inline assembly
#![feature(asm)]
// disable rust standard library
#![no_std]
// disables Rust runtime init,
#![no_main]
// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]
#![feature(unchecked_math)]

static mut I: u64 = 0;
const LOOP_CNT: u64 = 1 << 18;

fn plus() {
    unsafe {
        let mut z = I;
        z += 1;
        asm!("nop");
        I = z;
    };
}

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

/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() {
    use crate::mem::page_enabled;

    if !*page_enabled() {
        // setup page, page allocator, init thread pcb, jump to _start()
        *page_enabled() = true;
        println!("init page");
        println!("init page");
        crate::mem::init_page();
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

        for i in 0..16 {
            thread::new_thread(th, 0, "", 1);
        }

        println!("address of I = 0x{:08X}", unsafe {
            &I as *const _ as usize
        });
        loop {}
    }
}

extern "C" fn th(p: usize) {
    for _ in 0..LOOP_CNT {
        plus();
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{:#?}", _info);
    loop {}
}
