#[repr(packed)]
pub struct KernelCtx {
    pub ebp: u32,
    pub ebx: u32,
    pub edi: u32,
    pub esi: u32,
    pub eip: u32, // address of rust thread entry or the return address
    pub dum: u32, // return address
    pub func: u32,
    pub arg: u32,
}

#[repr(packed)]
pub struct IntCtx {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
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
}

