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

use crate::{mem::Pool, vga::{VGA_LOCK_REF, put_char}};

static mut I: u64 = 0;
const LOOP_CNT: u64 = 1 << 12;
const THREADS: u64 = 4;

fn plus() {
    unsafe {
        let mut z = I;
        z += 1;
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
        c_println!("_start = 0x{:08X}", _start as usize);
        c_println!("vga lock addr = 0x{:08X}", unsafe { &VGA_LOCK_REF as *const _ as usize });
        // c_println!("vga lock = {:?}", vga_lock().is_none());
        // c_println!("init page");
        asm::init();
        c_println!("asm init()");
        // setup page, page allocator, init thread pcb, jump to _start()
        *page_enabled() = true;
        c_println!("paged enabled = true");
        crate::thread::tss::init();
        c_println!("tss init");
        crate::mem::init_page();
    } else {
        c_println!("_start = 0x{:08X}", _start as usize);
        // load interrupt descriptor table
        int::init();

        // add main thread into list, register scheduler
        crate::thread::init();
        crate::init::init_locks();

        // increase interrupt frequency
        crate::timer::init();

        // enable interrupt
        asm::sti();

        crate::thread::new_thread(th_print_d, 0, "th0", 1);
        crate::thread::new_thread(th_print_d, 2, "th1", 1);
        bk!();
    }
}

extern "C" fn th_print_d(d: usize) {
        // println!("before init locked");
    loop {
        print!("{:02X} ", d);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{:#?}", _info);
    loop {}
}
