use core::fmt::Write;
use crate::fs::ide::PRIMARY_LEN;

pub mod ide;
mod ctl;

const DISKS_OFF: usize = 0x475;
const PT_OFF: usize = 446;

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

// partition table entry
#[repr(packed)]
#[derive(Debug)]
pub struct PTE {
    boot: u8,
    start_head: u8,
    start_sec: u8,
    start_cyl: u8,
    fs_type: u8,
    end_head: u8,
    end_sec: u8,
    end_cyl: u8,

    start_lba: u32,
    sec_n: u32,
}

pub trait BootSec {
    fn partition_table(&self) -> &[PTE];
}

impl BootSec for [u8] {
    fn partition_table(&self) -> &[PTE] {
        let p = unsafe { self.as_ptr().add(PT_OFF) as *const PTE };
        unsafe {
            core::slice::from_raw_parts(p, PRIMARY_LEN)
        }
    }
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