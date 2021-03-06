use crate::{asm, c_println, panic, print, println};
use crate::asm::SELECTOR_K_CODE;
use crate::thread::current_pcb;
use crate::thread::reg::IntCtx;
use crate::vga::{next_line, VGA_COL};

const ENTRY_SIZE: usize = 0x2f + 1;
pub const SYS_VEC: usize = 0x80;
const E_FLAGS_IF: u32 = 0x00000200;

// 32bit interrupt gate
const IDT_DESC_ATTR_DPL0: u8 = 1 << 7 | 0xe;
const IDT_DESC_ATTR_DPL3: u8 = 1 << 7 | 3 << 5 | 0xe;

static mut IDT_PTR: IdtPtr = IdtPtr { size: 0, off: 0 };
static mut IDT: [u64; SYS_VEC + 1] = [0; SYS_VEC + 1];
static mut HANDLERS: [usize; SYS_VEC + 1] = [0; SYS_VEC + 1];


pub static EXCEPTIONS: &[&'static str] = &[
    "#DE Divide Error",
    "#DB Debug Exception",
    "NMI Interrupt",
    "#BP Breakpoint Exception",
    "#OF Overflow Exception",
    "#BR BOUND Range Exceeded Exception",
    "#UD Invalid Opcode Exception",
    "#NM Device Not Available Exception",
    "#DF Double Fault Exception",
    "Coprocessor Segment Overrun",
    "#TS Invalid TSS Exception",
    "#NP Segment Not Present",
    "#SS Stack Fault Exception",
    "#GP General Protection Exception",
    "#PF Page-Fault Exception",
    "",
    "#MF x87 FPU Floating-Point Error",
    "#AC Alignment Check Exception",
    "#MC Machine-Check Exception",
    "#XF SIMD Floating-Point Exception",
];

// if return 0, recover from interrupt
// else switch current stack to another stack
extern "C" fn int_entry(esp: u32) {
    let ctx: &'static mut IntCtx = cst!(esp);
    let vec = ctx.vec;

    // if ctx.gs != 0 {
    //     let cur = current_pcb();
    // }
    // IntCtx::debug(ctx as *const _);
    // println!("m\n");
    // crate::vga::next_line();
    if vec < 20 {
        c_println!("EXCEPTION: {}", EXCEPTIONS[vec as usize]);
        loop {}
    }

    if (vec as usize) > SYS_VEC{
        return;
    }

    unsafe {
        let f = HANDLERS[vec as usize];
        if f == 0 {
            return;
        }

        let f: fn(ctx: &'static mut IntCtx) = core::mem::transmute(f);
        f(ctx);
    }
}

pub fn int_enabled() -> bool {
    let e_flags = crate::e_flags!();
    e_flags & E_FLAGS_IF != 0
}

pub fn register(vec: u16, handle: fn(int_ctx: &'static mut IntCtx)) {
    unsafe {
        HANDLERS[vec as usize] = handle as usize;
    }
}

fn idt() -> &'static mut [GateBits] {
    unsafe {
        let pp = IDT.as_ptr() as usize;
        core::slice::from_raw_parts_mut(pp as *mut _, SYS_VEC + 1)
    }
}

/// interrupt entries, cpu exception -> assembly code -> rust entry
fn int_entries() -> &'static [u32] {
    unsafe { core::slice::from_raw_parts(crate::asm::int_entries() as *const _, ENTRY_SIZE) }
}

/// location to register interrupt handler
fn int_rust() -> &'static mut u32 {
    unsafe { &mut *(crate::asm::int_rust() as *mut _) }
}

pub fn init() {
    // 1. register rust int handler
    let p = int_entry as usize;
    *int_rust() = p as u32;

    // 2. set up idt ptr
    unsafe {
        IDT_PTR = IdtPtr {
            size: ((SYS_VEC + 1) as u16) * 8 - 1,
            off: IDT.as_ptr() as usize as u32,
        }
    }

    // 3. init descriptors
    init_idt();

    // 4. init pic
    init_pic();

    // 5. call lidt
    unsafe { crate::asm::lidt((&IDT_PTR) as *const _ as usize) }
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

    out_b(PIC_M_DATA, 0xf8);
    out_b(PIC_S_DATA, 0xbf);
}

/// initialize interrupt descriptor table
fn init_idt() {
    let entries = int_entries();
    let t = idt();

    for i in 0..ENTRY_SIZE {
        t[i] = GateBits::new(entries[i], IDT_DESC_ATTR_DPL0)
    }

    t[SYS_VEC] = GateBits::new(asm::sys() as u32, IDT_DESC_ATTR_DPL3);
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
            selector: SELECTOR_K_CODE,
            reserved: 0,
            attr,
            off_high: ((entry & 0xffff0000) >> 16) as u16,
        }
    }
}

pub fn enable_int() -> bool {
    let old = int_enabled();
    if !old {
        asm::sti();
    }
    old
}

pub fn disable_int() -> bool {
    let old = int_enabled();
    if old {
        asm::cli();
    }
    old
}

pub fn set_int(enabled: bool) {
    if enabled {
        enable_int();
    } else {
        disable_int();
    }
}
