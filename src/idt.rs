pub type HandleFn = fn() -> !;

static mut IDTR: IDTR = IDTR { size: 0, off: 0 };
static mut IDT: [u64; 1] = [0];

#[repr(packed)]
struct IDTR {
    size: u16,
    off: u32,
}


struct Gate {
    handle: HandleFn,
    selector: u16,
}

#[repr(packed)]
struct GateBits {
    off_0: u16,
    selector: u16,
    reserved: u8,
    mask: u8,
    off_1: u16,
}

