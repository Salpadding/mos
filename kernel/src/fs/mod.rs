mod ide;

const DISKS_OFF: usize = 0x475;

pub fn disks() -> u8 {
    unsafe {
        *(DISKS_OFF as * const u8)
    }
}