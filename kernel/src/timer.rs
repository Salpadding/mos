use crate::asm;

const COUNTER0_PORT: u16 = 0x40;
const PIT_CONTROL_PORT: u16 = 0x43;
const READ_WRITE_LATCH: u8 = 3;
const COUNTER_MODE: u8 = 2;
const IRQ0_FREQUENCY: u32 = 10000;
const INPUT_FREQUENCY: u32 = 1193180;
const COUNTER0_VALUE: u32 = INPUT_FREQUENCY / IRQ0_FREQUENCY;

pub fn init() {
    asm::out_b(PIT_CONTROL_PORT, READ_WRITE_LATCH << 4 | COUNTER_MODE << 1);
    asm::out_b(COUNTER0_PORT, (COUNTER0_VALUE & 0xff) as u8);
    asm::out_b(COUNTER0_PORT, (COUNTER0_VALUE >> 8 & 0xff) as u8);
}
