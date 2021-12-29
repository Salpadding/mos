use crate::println;

pub const ASM_BUF_OFF: usize = 4096;
pub const ASM_BUF_LEN: usize = 64;
pub const REG_CTX_LEN: usize = ASM_BUF_LEN;
pub const ASM_API_OFF: usize = ASM_BUF_OFF + ASM_BUF_LEN * 4;
pub const KERNEL_ENTRY: usize = 1 << 20;

type AsmApi = extern "C" fn();
type AsmBuf = &'static mut [u32];

// get register context, for
pub fn reg_ctx() -> &'static mut [u32] {
    asm_buf()
}

macro_rules! ix {
    ($f: ident) => {
        fn $f(&mut self) -> &mut u32;
    };
}

macro_rules! mx {
    ($f: ident, $i: expr) => {
        fn $f(&mut self) -> &mut u32 {
            &mut self[$i]
        }
    };
}

pub trait IntCtx {
    ix!(eax);
    ix!(ebx);
    ix!(ecx);
    ix!(edx);
    ix!(esp);
    ix!(ebp);
    ix!(esi);
    ix!(edi);
    ix!(es);
    ix!(cs);
    ix!(ds);
    ix!(fs);
    ix!(gs);
    ix!(eip);
    ix!(err_code);
    ix!(e_flags);
    ix!(vec);

    fn reset_general(&mut self);

    fn print(&mut self) {
        let eax = *self.eax();
        let ebx = *self.ebx();
        let ecx = *self.ecx();
        let edx = *self.edx();
        let esp = *self.esp();
        let ebp = *self.ebp();
        let esi = *self.esi();
        let edi = *self.edi();
        let ds = *self.ds();
        let es = *self.es();
        let fs = *self.fs();
        let gs = *self.gs();
        let cs = *self.cs();
        let eip = *self.eip();
        println!("eax = 0x{:08X} ebx = 0x{:08X} ecx = 0x{:08X} edx = 0x{:08X}", eax, ebx, ecx, edx);
        println!("esp = 0x{:08X} ebp = 0x{:08X} esi = 0x{:08X} edi = 0x{:08X}", esp, ebp, esi, edi);
        println!("ds = 0x{:08X}  es  = 0x{:08X} fs  = 0x{:08X} gs  = 0x{:08X}", ds, es, fs, gs);
        println!("cs = 0x{:08X}  eip = 0x{:08X}", cs, eip);
    }
}

impl IntCtx for [u32] {
    mx!(eax, ASM_BUF_LEN - 1);
    mx!(ecx, ASM_BUF_LEN - 2);
    mx!(edx, ASM_BUF_LEN - 3);
    mx!(ebx, ASM_BUF_LEN - 4);
    mx!(esp, ASM_BUF_LEN - 5);
    mx!(ebp, ASM_BUF_LEN - 6);
    mx!(esi, ASM_BUF_LEN - 7);
    mx!(edi, ASM_BUF_LEN - 8);
    mx!(ds, ASM_BUF_LEN - 9);
    mx!(es, ASM_BUF_LEN - 10);
    mx!(fs, ASM_BUF_LEN - 11);
    mx!(gs, ASM_BUF_LEN - 12);

    mx!(cs, 3);
    mx!(eip, 2);
    mx!(err_code, 1);
    mx!(e_flags, 4);
    mx!(vec, 0);

    fn reset_general(&mut self) {
        self[ASM_BUF_LEN - 8..ASM_BUF_LEN].fill(0);
    }
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

pub fn gdt() -> &'static mut GdtPtr {
    let p = api_call(methods::GDT_PTR, &[]);
    unsafe { &mut *(p as *mut _) }
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
    unsafe { asm!("sti"); }
}

pub fn cli() {
    unsafe { asm!("cli"); }
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
