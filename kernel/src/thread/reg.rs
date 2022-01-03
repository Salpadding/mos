use crate::println;

#[repr(packed)]
#[derive(Debug)]
pub struct KernelCtx {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub dum_esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub eip: u32,
    // address of rust thread entry or the return address
    pub dum_ret: u32,
    // return address
    pub func: u32,
    pub arg: u32,
}

static INT_CTX_NAMES: &[&'static str] = &[
    "vec", "edi", "esi", "ebp", "dum",
    "ebx", "edx", "ecx", "eax",
    "gs", "fs", "es", "ds",
    "e_code", "eip", "cs", "e_flags",
    "esp", "ss"
];

#[repr(packed)]
#[derive(Debug)]
pub struct IntCtx {
    pub vec: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub dum: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,

    pub e_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub e_flags: u32,
    pub esp: u32,
    pub ss: u32,
}

impl IntCtx {
    pub fn debug(ctx: *const Self) {
        let p = ctx as *const u32;

        for i in 0..INT_CTX_NAMES.len() {
            let n = unsafe { *p.add(i) };
            println!("{} = 0x{:08X}", INT_CTX_NAMES[i], n);
        }
    }
}


