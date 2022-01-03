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
