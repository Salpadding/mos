use rlib::gdt::gdt;
use crate::asm::SELECTOR_TSS;
use crate::println;

pub const TSS_LEN: usize = 27;
pub const TSS_SIZE: usize = TSS_LEN * 4;
pub const TSS_BOUND: usize = TSS_SIZE - 1;

pub static mut TSS_DATA: [u32; TSS_LEN] = [0u32; TSS_LEN];

macro_rules! m {
    ($f: ident, $fm: ident) => {
        fn $f(&self) -> u32;
        fn $fm(&mut self) -> &mut u32;
    };
}

macro_rules! im {
    ($f: ident, $fm: ident, $i: expr) => {
        fn $f(&self) -> u32 { self[$i] }
        fn $fm(&mut self) -> &mut u32 { &mut self[$i] }
    };
}

pub trait TSS {
    m!(back_link, back_link_mut);
    m!(esp0, esp0_mut);
    m!(ss0, ss0_mut);
    // esp1
    // ss1
    // esp2
    // ss2
    // cr3
    // eip
    // e_flags
    // eax
    // ecx
    // edx
    // ebx

}

impl TSS for [u32] {
    im!(back_link, back_link_mut, 0);
    im!(esp0, esp0_mut, 1);
    im!(ss0, ss0_mut, 2);
}

const GDT_OFF: usize = 0x800 + 8;
const GDT_LEN: usize = 8;

pub fn init() {

    // 0, 1, 2 is predefined
   let gdt = gdt(GDT_OFF, GDT_LEN);

    // 3 is user code
    gdt[3] = rlib::gdt::user_code();
    // 4 is user data
    gdt[4] = rlib::gdt::user_data();

    let mut bd = rlib::gdt::GdtBuilder::default();
    gdt[5] = bd.present(true).base(unsafe { TSS_DATA.as_ptr() as usize as u32 })
        .limit(TSS_BOUND as u32)
        .access(true)
        .privilege(0)
        .system(true)
        .lim_4k(true)
        .executable(true).build();


    println!("gdt ptr = {}", crate::asm::gdt());
    println!("{:?}", gdt);

    // load gdt and tss
    unsafe {
        asm!("lgdt [{}]", in(reg) crate::asm::gdt());
        let v: u32 = SELECTOR_TSS as u32;
        asm!("ltr [{}]", in(reg) &v as *const _ as usize);
    }
}