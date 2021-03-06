use core::fmt;
use core::fmt::Write;

pub const ASM_BUF_OFF: usize = 4096;
pub const ASM_BUF_LEN: usize = 64;
pub const REG_CTX_LEN: usize = ASM_BUF_LEN;
pub const ASM_API_OFF: usize = ASM_BUF_OFF + ASM_BUF_LEN * 4;
pub const KERNEL_ENTRY: usize = 1 << 20;

pub const SELECTOR_K_CODE: u16 = 1 << 3;
pub const SELECTOR_K_DATA: u16 = 2 << 3;
pub const SELECTOR_U_CODE: u16 = 3 << 3 | 3;
pub const SELECTOR_U_DATA: u16 = 4 << 3 | 3;
pub const SELECTOR_TSS: u16 = 5 << 3;

type AsmApi = extern "C" fn();
type AsmBuf = &'static mut [u32];

static mut SWITCH_ADDR: usize = 0;
static mut INT_EXIT: usize = 0;
static mut SYS: usize = 0;

use rlib::alloc_static;
use crate::thread::sync::SpinLock;

alloc_static!(LOCK, lock, SpinLock);

pub fn int_exit() -> usize {
    unsafe { INT_EXIT }
}

pub fn sys() -> usize { unsafe { SYS } }

pub fn init() {
    unsafe {
        SWITCH_ADDR = api_call(methods::SWITCH_ADDR, &[]) as usize;
        INT_EXIT = api_call(methods::INT_EXIT, &[]) as usize;
        SYS = api_call(methods::SYS, &[]) as usize;
    }
}

pub fn switch(cur: usize, next: usize) {
    let f: extern "C" fn(cur: u32, next: u32) = unsafe { core::mem::transmute(SWITCH_ADDR) };
    f(cur as u32, next as u32);
}

#[macro_export]
macro_rules! e_flags {
    () => {
        {
            let x: u32;
            unsafe {
                asm!("pushfd", "pop {}", out(reg) x);
            }
            x
        }
    };
}

pub fn asm_buf() -> AsmBuf {
    unsafe { core::slice::from_raw_parts_mut(ASM_BUF_OFF as *mut _, ASM_BUF_LEN) }
}

pub fn asm_api() -> AsmApi {
    unsafe { core::mem::transmute(ASM_API_OFF) }
}

mod methods {
    pub const GDT_PTR: u32 = 0;
    pub const INT_ENTRIES_OFF: u32 = 1;
    pub const INT_RUST_OFF: u32 = 2;
    pub const MEM_SZ: u32 = 3;
    pub const SWITCH_ADDR: u32 = 4;
    pub const INT_EXIT: u32 = 5;
    pub const SYS: u32 = 6;
}

fn api_call(method: u32, args: &[u32]) -> u32 {
    let buf = asm_buf();
    buf[0] = method;
    buf[1..(1 + args.len())].copy_from_slice(args);
    let api = asm_api();
    api();
    buf[0]
}

pub fn memory_size() -> u32 {
    api_call(methods::MEM_SZ, &[])
}

pub fn out_b(port: u16, b: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") b) };
}

pub fn in_b(port: u16) -> u8 {
    let r: u8;
    unsafe { asm!("in al, dx", out("al") r, in("dx") port) };
    r
}

pub fn out_sw(port: u16, buf: &[u16]) {
    unsafe {
        asm!(
        "cld",
        "rep outsw",
        in("dx") port,
        in("ecx")  buf.len(),
        in("edi")  buf.as_ptr() as usize
        )
    }
}

pub fn in_sw(port: u16, buf: &mut [u16]) {
    unsafe {
        asm!(
        "cld",
        "rep insw",
        in("dx") port,
        in("ecx")  buf.len(),
        in("edi")  buf.as_ptr() as usize
        )
    }
}

pub fn gdt() -> u32 {
    let p = api_call(methods::GDT_PTR, &[]);
    p
}

pub fn lidt(addr: usize) {
    unsafe {
        asm!("lidt [{}]", in(reg) addr);
    }
}

/// setup page, new stack top, then jump to callback
pub fn page_jmp(pde_start: usize, new_stack: usize, cb: usize) {
    unsafe {
        let mut cr0: u32;
        asm!("mov {}, cr0", out(reg) cr0);
        cr0 |= 1 << 31;
        asm!("mov cr3, {0}", in(reg) pde_start);
        asm!("mov cr0, {}", in(reg) cr0);
        asm!("mov ebp, {0}", "mov esp, ebp", in(reg) new_stack);
        asm!("jmp {0}", in(reg) cb);
    }
}

pub fn int_entries() -> usize {
    api_call(methods::INT_ENTRIES_OFF, &[]) as usize
}

pub fn int_rust() -> usize {
    api_call(methods::INT_RUST_OFF, &[]) as usize
}

pub fn sti() {
    unsafe {
        asm!("sti");
    }
}

pub fn cli() {
    unsafe {
        asm!("cli");
    }
}

#[repr(packed)]
pub struct GdtPtr {
    pub gdt_bound: u16,
    pub gdt_base: usize,
}

impl GdtPtr {
    pub fn gdt(&mut self) -> &'static mut [u64] {
        let sz = (self.gdt_bound + 1) as usize;
        let n = sz / 8;
        unsafe { core::slice::from_raw_parts_mut(self.gdt_base as *mut _, n) }
    }
}

const PORT: u16 = 0x3f8;

pub fn out_c(c: u8) {
    while crate::asm::in_b(PORT + 5) & 0x20 == 0 {}
    out_b(PORT, c);
}

pub fn out_s(s: &str) {
    for c in s.as_bytes() {
        out_c(*c);
    }
}

pub struct Writer {}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        out_s(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut w = Writer {};
    w.write_fmt(args);
}


#[macro_export]
macro_rules! c_print {
    ($($arg:tt)*) => ($crate::asm::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! c_println {
    () => ($crate::asm::out_b(b'\n'));
    ($($arg:tt)*) => {
        $crate::asm::_print(format_args!($($arg)*));
        $crate::asm::out_c(b'\n');
    };
}