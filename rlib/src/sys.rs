use crate::sys::NR::WRITE;

pub mod NR {
    pub const GET_PID: u32 = 0;
    pub const WRITE: u32 = 1;
}


#[inline]
pub fn call_0(n: u32) -> u32 {
    let mut ret: u32 = n;
    unsafe {
        asm!("int 80h", inout("eax") ret);
    }
    ret
}

#[inline]
pub fn call_1(n: u32, a: u32) -> u32 {
    let mut ret: u32 = n;
    unsafe {
        asm!("int 80h", inout("eax") ret, in("ebx") a);
    }
    ret
}

#[inline]
pub fn call_2(n: u32, a: u32, b: u32) -> u32 {
    let mut ret: u32 = n;
    unsafe {
        asm!("int 80h", inout("eax") ret, in("ebx") a, in("ecx") b);
    }
    ret
}

#[inline]
pub fn call_3(n: u32, a: u32, b: u32, c: u32) -> u32 {
    let mut ret: u32 = n;
    unsafe {
        asm!("int 80h", inout("eax") ret, in("ebx") a, in("ecx") b, in("edx") c);
    }
    ret
}

pub fn write(p: *const u8, len: usize) {
    call_2(WRITE as u32, p as usize as u32, len as u32);
}

