use rlib::gdt::gdt;

use crate::asm::{SELECTOR_K_DATA, SELECTOR_TSS};
use crate::println;

pub const TSS_LEN: usize = 27;
pub const TSS_SIZE: usize = TSS_LEN * 4;
pub const TSS_BOUND: usize = TSS_SIZE - 1;

pub static mut TSS_DATA: [u32; TSS_LEN] = [0u32; TSS_LEN];

fn tss() -> &'static mut [u32] {
    unsafe {
        &mut TSS_DATA
    }
}

macro_rules! m {
    ($f: ident, $fm: ident, $i: expr) => {
        fn $f(&self) -> u32 {
            self.slice()[$i]
        }
        fn $fm(&mut self) -> &mut u32 {
            &mut self.slice_mut()[$i]
        }
    };
}

// Task state segment, record stack pointer when switching from user to kernel 
pub trait TSS {
    fn slice(&self) -> &[u32];
    fn slice_mut(&mut self) -> &mut [u32];
    m!(back_link, back_link_mut, 0);
    m!(esp0, esp0_mut, 1);
    m!(ss0, ss0_mut, 2);
    m!(esp1, esp1_mut, 3);
    m!(ss1, ss1_mut, 4);
    m!(esp2, esp2_mut, 5);
    m!(ss2, ss2_mut, 6);
    m!(cr3, cr3_mut, 7);
    m!(eip, eip_mut, 8);
    m!(e_flags, e_flags_mut, 9);
    m!(eax, eax_mut, 10);
    m!(ecx, ecx_mut, 11);
    m!(edx, edx_mut, 12);
    m!(ebx, ebx_mut, 13);
    m!(esp, esp_mut, 14);
    m!(ebp, ebp_mut, 15);
    m!(esi, esi_mut, 16);
    m!(edi, edi_mut, 17);
    m!(es, es_mut, 18);
    m!(cs, cs_mut, 19);
    m!(ss, ss_mut, 20);
    m!(ds, ds_mut, 21);
    m!(fs, fs_mut, 22);
    m!(gs, gs_mut, 23);
    m!(ldt, ldt_mut, 24);
    m!(trace, trace_mut, 25);
    m!(io_base, io_base_mut, 26);
}

impl TSS for [u32] {
    fn slice(&self) -> &[u32] {
        &self
    }

    fn slice_mut(&mut self) -> &mut [u32] {
        self
    }
}

const GDT_OFF: usize = 0x800 + 8;
const GDT_LEN: usize = 8;

pub fn esp0() -> &'static mut u32 {
    tss().esp0_mut()
}

pub fn init() {
    let tss = tss();
    *tss.ss0_mut() = SELECTOR_K_DATA as u32;
    *tss.io_base_mut() = (TSS_LEN * 4) as u32;

    // 0, 1, 2 is predefined
    let gdt = gdt(GDT_OFF, GDT_LEN);

    // 3 is user code
    gdt[3] = rlib::gdt::user_code();
    // 4 is user data
    gdt[4] = rlib::gdt::user_data();

    let mut bd = rlib::gdt::GdtBuilder::default();
    gdt[5] = bd
        .present(true)
        .base(unsafe { TSS_DATA.as_ptr() as usize as u32 })
        .limit(TSS_BOUND as u32)
        .access(true)
        .privilege(0)
        .system(true)
        .lim_4k(true)
        .executable(true)
        .build();


    // load gdt and tss
    unsafe {
        asm!("lgdt [{}]", in(reg) crate::asm::gdt());
        asm!("ltr ax", in("ax") SELECTOR_TSS);
    }
}
