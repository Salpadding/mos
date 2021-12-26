use crate::asm::methods::LIDT;
use crate::println;

// loader off + pad + gdt
pub const ASM_BUF_OFF: usize = 0x800 + 8 + 32;
pub const ASM_BUF_LEN: usize = 4;
pub const ASM_API_OFF: usize = ASM_BUF_OFF + ASM_BUF_LEN * 4;
pub const KERNEL_ENTRY: usize = 1 << 20;

type AsmApi = extern "C" fn();
type AsmBuf = &'static mut [u32];

pub fn asm_buf() -> AsmBuf {
    unsafe { core::slice::from_raw_parts_mut(ASM_BUF_OFF as *mut _, ASM_BUF_LEN) }
}

pub fn asm_api() -> AsmApi {
    unsafe { core::mem::transmute(ASM_API_OFF) }
}

mod methods {
    pub const ECHO: u32 = 0;
    pub const GDT_PTR: u32 = 1;
    pub const LIDT: u32 = 2;
    pub const PAGE_ENABLED: u32 = 3;
    pub const PAGE_SETUP: u32 = 4;
    pub const INT_ENTRIES_OFF: u32 = 5;
    pub const INT_RUST_OFF: u32 = 6;
    // division by assembly
    pub const DIV: u32 = 7;
    pub const CALLER: u32 = 8;
    pub const OUT_B: u32 = 9;
    pub const OUT_SW: u32 = 10;
    pub const IN_B: u32 = 11;
    pub const IN_SW: u32 = 12;
    pub const STI: u32 = 13;
    pub const MEM_SZ: u32 = 14;
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
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") b)
    };
}

pub fn in_b(port: u16) -> u8 {
    let r: u8;
    unsafe {
        asm!("in al, dx", out("al") r, in("dx") port)
    };
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

pub fn echo(x: u32) -> u32 {
    api_call(methods::ECHO, &[x])
}

pub fn gdt() -> &'static mut GdtPtr {
    let p = api_call(methods::GDT_PTR, &[]);
    unsafe { &mut *(p as *mut _) }
}

pub fn lidt(addr: usize) {
    unsafe {
        asm!("lidt [{}]", in(reg) addr);
    }
    // api_call(methods::LIDT, &[addr as u32]);
}

pub fn reset_page(pde_start: usize, new_stack: usize, cb: usize)  {
    unsafe {
        let mut cr0: u32;
        asm!("mov {}, cr0", out(reg) cr0);
        cr0 |= 1 << 31;
        asm!(
        "mov cr3, {0}",
        "mov ebp, {1}",
        "mov esp, ebp",
        in(reg) pde_start,
        in(reg) new_stack
        );
        asm!("mov cr0, {}", in(reg) cr0);
        asm!("jmp {}", in(reg) cb);
    }
}

pub fn int_entries() -> usize {
    api_call(methods::INT_ENTRIES_OFF, &[]) as usize
}

pub fn int_rust() -> usize {
    api_call(methods::INT_RUST_OFF, &[]) as usize
}

pub fn div(x: u32, y: u32) -> u32 {
    api_call(methods::DIV, &[x, y])
}

pub fn caller() -> u32 {
    api_call(methods::CALLER, &[])
}

pub fn sti() {
    unsafe { asm!("sti"); }
}

#[repr(packed)]
pub struct GdtPtr {
    pub gdt_bound: u16,
    gdt_base: usize,
}

impl GdtPtr {
    pub fn gdt(&mut self) -> &'static mut [u64] {
        let sz = (self.gdt_bound + 1) as usize;
        let n = sz / 8;
        unsafe { core::slice::from_raw_parts_mut(self.gdt_base as *mut _, n) }
    }
}
