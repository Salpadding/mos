use core::fmt::Write;

pub mod ide;
mod ctl;

const DISKS_OFF: usize = 0x475;

pub fn disks() -> u8 {
    unsafe {
        *(DISKS_OFF as * const u8)
    }
}

pub trait DiskInfo {
    fn write_sn<T: core::fmt::Write>(&self, t: &mut T);
}

impl DiskInfo for [u8] {
    fn write_sn<T: Write>(&self, t: &mut T) {
        let sn_start: usize = 20;
        let sn_len: usize = 20;
        let z = &self[sn_start..sn_start+sn_len];

        for i in 0..z.len() / 2 {
            t.write_char(z[i * 2 + 1 ] as char);
            t.write_char(z[i * 2] as char);
        }
    }
}