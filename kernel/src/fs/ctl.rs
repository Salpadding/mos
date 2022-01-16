use crate::asm::out_b;
use crate::fs::ide::IdeChannel;
use super::ide::Disk;

// bits of status register
pub const BIT_STAT_BSY: u8 = 0x80; // drive is busy
pub const BIT_STAT_DRDY: u8 = 0x40; // drive is ready
pub const BIT_STAT_DRQ:  u8 = 0x8; // ready for data request

// bits of device register
pub const BIT_DEV_MBS: u8 = 0xa0;
pub const BIT_DEV_LBA: u8 = 0x40;
pub const BIT_DEV_DEV: u8 = 0x10;

// bits of drive operation
pub const CMD_ID: u8 = 0xec; // identify command
pub const CMD_READ_SEC: u8 = 0x20; // read section command
pub const CMD_WRITE_SEC: u8 = 0x30; // write section command




