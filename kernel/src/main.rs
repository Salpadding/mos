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

use vga::{put_char, puts};

use crate::mem::Pool;
use crate::vga::vga_lock;

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
mod int;
mod mem;
mod thread;
mod timer;
mod vga;
mod init;

/// The name **must be** `_start`, otherwise the compiler doesn't output anything
/// to the object file. I don't know why it is like this.
#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn _start() {
    use crate::mem::page_enabled;

    if !*page_enabled() {
        crate::init::init_statics();
        asm::init();
        // setup page, page allocator, init thread pcb, jump to _start()
        *page_enabled() = true;
        println!("init page");
        crate::mem::init_page();
    } else {
        // load interrupt descriptor table
        int::init();

        // add main thread into list, register scheduler
        crate::thread::init();
        unsafe { vga::THREAD_INIT = true };

        println!("thread init success");

        // increase interrupt frequency
        crate::timer::init();
        println!("timer init success");
        println!("sti success");

        crate::thread::new_thread(th_print_d, 0, "th0", 1);
        // crate::thread::new_thread(th_print_d, 2, "th1", 1);

        // enable interrupt
        asm::sti();

        let lock = vga_lock();
        loop {
            lock.lock();
            // println!("locked success");
            // print!("01");
            unsafe { asm!("nop"); }
            lock.unlock();
            // println!("unlock success");
        }
    }
}

static s1: &'static str = "argA ";
static s2: &'static str = "argB ";

extern "C" fn th_print_d(d: usize) {
    let lock = vga_lock();
    loop {
        // println!("before init locked");
        lock.lock();
        // println!("init locked");
        unsafe { asm!("nop"); }
        // print!("{:02X}", d);
        // println!("before init unlock");
        lock.unlock();
        // println!("init unlocked");
    }
}

extern "C" fn th_print(p: usize) {
    loop {
        let mut chars = p as *const u8;

        loop {
            unsafe {
                let c = *chars;
                chars = chars.add(1);

                put_char(c);

                if c == b' ' {
                    break;
                }
            }
        }
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{:#?}", _info);
    loop {}
}
