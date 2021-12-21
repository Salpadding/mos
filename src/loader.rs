pub const LOADER_API_OFF: usize = 0x800 + 8;

mod methods {
    const GDT_PTR_ADDR: u32 = 1;
    const LIDT: u32 = 2;
}

type LoaderApi = extern "C" fn(method: u32, p0: u32) -> u32;

pub fn loader_api() -> LoaderApi {
    unsafe {
        core::mem::transmute(LOADER_API_OFF)
    }
}

pub fn gdt() -> &'static mut GdtPtr {
    let api = loader_api();
    let x = api(1, 0) as usize;
    unsafe { &mut *(x as *mut _) }
}

pub fn lidt(addr: usize)  {
    let api = loader_api();
    api(2, addr as u32);
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
