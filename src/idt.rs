use crate::println;

const ENTRY_SIZE: usize = 33;
const SELECTOR_CODE: u16 = 1 << 3;

// 32bit interrupt gate
const IDT_DESC_ATTR_DPL0: u8 =  (1 << 7) | 0xe;


pub type HandleFn = fn() -> !;

static mut IDT_PTR: IdtPtr = IdtPtr { size: 0, off: 0 };
static mut IDT: [u64; ENTRY_SIZE] = [0; ENTRY_SIZE];

pub extern "C" fn int_entry() {
    let vec = crate::asm::asm_buf()[0];


    if vec == 0 {
        println!("divided by zero!");
        loop {}
    }
}

fn idt() -> &'static mut [GateBits] {
    unsafe {
        let pp = IDT.as_ptr() as usize;
        core::slice::from_raw_parts_mut(
            pp as *mut _,
            ENTRY_SIZE,
        )
    }
}

pub fn int_entries() -> &'static [u32] {
    unsafe {
        core::slice::from_raw_parts(
            crate::asm::int_entries() as *const _,
            ENTRY_SIZE,
        )
    }
}

fn int_rust() -> &'static mut u32 {
    unsafe {
        &mut *(crate::asm::int_rust() as *mut _)
    }
}

pub fn init_all() {
    // 1. register rust int handler
    let p = int_entry as usize;
    *int_rust() = p as u32;

    // 2. set up idt ptr
    unsafe {
        IDT_PTR = IdtPtr {
            size: (ENTRY_SIZE as u16) * 8 - 1,
            off: IDT.as_ptr() as usize as u32,
        }
    }

    // 3. init descriptors
    init_idt();

    // 4. init pic
    init_pic();

    // 5. call lidt
    unsafe {
        crate::asm::lidt((&IDT_PTR) as *const _ as usize)
    }

}

const PIC_M_CTRL: u16 = 0x20;
const PIC_M_DATA: u16 = 0x21;
const PIC_S_CTRL: u16 = 0xa0;
const PIC_S_DATA: u16 = 0xa1;

fn init_pic() {
    use crate::asm::out_b;

    out_b(PIC_M_CTRL, 0x11);
    out_b(PIC_M_DATA, 0x20);
    out_b(PIC_M_DATA, 0x04);
    out_b(PIC_M_DATA, 0x01);


    out_b(PIC_S_CTRL, 0x11);
    out_b(PIC_S_DATA, 0x28);
    out_b(PIC_S_DATA, 0x02);
    out_b(PIC_S_DATA, 0x01);

    out_b(PIC_M_DATA, 0xfe);
    out_b(PIC_S_DATA, 0xff);

}

fn init_idt() {
    let entries = int_entries();
    let t = idt();

    for i in 0..ENTRY_SIZE {
        t[i] = GateBits::new(entries[i], IDT_DESC_ATTR_DPL0)
    }
}

#[repr(packed)]
struct IdtPtr {
    size: u16,
    off: u32,
}


#[repr(packed)]
#[derive(Default, Copy, Clone)]
struct GateBits {
    off_low: u16,
    selector: u16,
    reserved: u8,
    attr: u8,
    off_high: u16,
}

impl GateBits {
    fn new(entry: u32, attr: u8) -> Self {
        Self {
            off_low: (entry & 0xffff) as u16,
            selector: SELECTOR_CODE,
            reserved: 0,
            attr,
            off_high: ((entry & 0xffff0000) >> 16) as u16
        }
    }
}

