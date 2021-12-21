use crate::asm::methods::LIDT;
use crate::println;

pub const ASM_API_OFF: usize = 0x800 + 8;

type AsmApi = extern "C" fn(method: u32, p0: u32) -> u32;


pub fn asm_api() -> AsmApi {
    unsafe {
        core::mem::transmute(ASM_API_OFF)
    }
}

mod methods {
    pub const GDT_PTR: u32 = 1;
    pub const LIDT: u32 = 2;
    pub const PAGE_ENABLED: u32 = 3;
    pub const PAGE_SETUP: u32 = 4;
}

pub fn gdt() -> &'static mut GdtPtr {
    let api = asm_api();
    let x = api(methods::GDT_PTR, 0) as usize;
    unsafe { &mut *(x as *mut _) }
}

pub fn lidt(addr: usize)  {
    let api = asm_api();
    api(LIDT, addr as u32);
}

pub fn page_enabled() -> bool {
    let api = asm_api();
    api(methods::PAGE_ENABLED, 0) != 0
}

pub fn page_setup() -> ! {
    let api = asm_api();
    let r = api(methods::PAGE_SETUP, 127);
    println!("{}", r);
    loop {

    }
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
        unsafe {
            core::slice::from_raw_parts_mut(
                self.gdt_base as *mut _,
                n,
            )
        }
    }
}

