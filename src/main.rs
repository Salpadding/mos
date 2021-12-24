use std::io::Read;

const GDT_OFF: usize = 8;
const GDT_LEN: usize = 4;

fn main() {
    println!("hello world")
}


#[repr(packed)]
#[derive(Default)]
struct GD {
    lim_low: u16,
    base_low: u16,
    base_mid: u8,
    acc: u8,
    lim_flags: u8,
    base_high: u8,
}

const MODE_REAL: u8 = 0;
const MODE_PROTECT: u8 = 1 << 6;
const MODE_LONG: u8 = 1 << 5;
const PRI_KERNEL: u8 = 0;
const PRI_USER: u8 = 3;

impl GD {
    fn new(limit: u32, base: u32, rw: bool, ex: bool, sys: bool, mode: u8, pri: u8, scale_4k: bool) -> Self {
        let mut acc: u8 = 1 << 7;
        if rw {
            acc = acc | (1 << 1);
        }
        if ex {
            acc = acc | (1 << 3);
        }
        if !sys {
            acc = acc | (1 << 4);
        }
        acc = acc | ((pri & 3) << 5);

        let mut fl = ((limit & 0xf0000) >> 16) as u8;
        fl = fl | mode;
        if scale_4k {
            fl = fl | (1 << 7);
        }

        Self {
            lim_low: (limit & 0xffff) as u16,
            base_low: (base & 0xffff) as u16,
            base_mid: ((base & 0xff0000) >> 16) as u8,
            acc,
            lim_flags: fl,
            base_high: (base >> 24) as u8,
        }
    }
}

fn set_gdt() {
    let mut file = std::fs::File::open("build/loader.bin").unwrap();
    let file_len = file.metadata().unwrap().len();
    let mut v = Vec::with_capacity(file_len as usize);
    file.read_to_end(&mut v).unwrap();

    let gdt: &mut [GD] = unsafe {
        let p = v.as_ptr().offset(GDT_OFF as isize);
        core::slice::from_raw_parts_mut(p as *mut _, GDT_LEN)
    };

    gdt[0] = GD::default();
    gdt[1] = GD::new(0, 0, true, true, false, MODE_PROTECT, PRI_KERNEL, true);
}