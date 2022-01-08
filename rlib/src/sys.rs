#[inline]
pub fn call_0(n: u32) -> u32 {
    let mut ret: u32 = n;
    unsafe {
        asm!("int 80h", inout("eax") ret);
    }
    ret
}