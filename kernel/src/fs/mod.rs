use core::fmt::Write;

pub mod ide;
mod ctl;

const DISKS_OFF: usize = 0x475;

pub fn disks() -> u8 {
    unsafe {
        *(DISKS_OFF as *const u8)
    }
}

pub trait DiskInfo {
    fn write_as_be<T: core::fmt::Write>(&self, t: &mut T, off: usize, len: usize);
    fn write_sn<T: core::fmt::Write>(&self, t: &mut T) {
        self.write_as_be(t, 20, 20);
    }
    fn write_md<T: core::fmt::Write>(&self, t: &mut T) {
        self.write_as_be(t, 27 * 2, 40);
    }

    fn sectors(&self) -> usize;
}

impl DiskInfo for [u8] {
    fn write_as_be<T: Write>(&self, t: &mut T, off: usize, len: usize) {
        let z = &self[off..off + len];
        for i in 0..z.len() / 2 {
            let c = z[i * 2 + 1];
            if c == 0 {
                break;
            }
            t.write_char(c as char);
            let c = z[i * 2];
            if c == 0 {
                break;
            }
            t.write_char(c as char);
        }
    }

    fn sectors(&self) -> usize {
        unsafe { *(self.as_ptr().add(60 * 2) as *const u32) as usize }
    }
}