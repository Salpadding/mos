use rlib::bitmap::Bitmap;

use crate::{asm, println};

const USED_MEM: usize = 8 * 1024 * 1024;
const KERNEL_MEM: usize = 4 * 1024 * 1024;
const PAGE_SIZE: usize = 4 * 1024;

/// 128kb bit map
const BIT_MAP_SIZE: usize = (4 * 1024 * 1024 * 1024 / PAGE_SIZE as u64 / 8) as usize;

static mut BIT_MAP_DATA: [u8; BIT_MAP_SIZE] = [0u8; BIT_MAP_SIZE];

/// page bitmap for kernel space
static mut KERNEL_BIT_MAP: &'static mut [u8] = unsafe { &mut BIT_MAP_DATA };
static mut KERNEL_P_START: usize = 0;

/// page bitmap for user space
static mut USER_BIT_MAP: &'static mut [u8] = unsafe { &mut BIT_MAP_DATA };
static mut USER_P_START: usize = 0;

struct PagePool {
    bitmap: &'static mut [u8],
    p_start: &'static mut usize,
}

fn kernel_pool() -> PagePool {
    unsafe {
        PagePool {
            bitmap: KERNEL_BIT_MAP,
            p_start: &mut KERNEL_P_START,
        }
    }
}

fn user_pool() -> PagePool {
    unsafe {
        PagePool {
            bitmap: USER_BIT_MAP,
            p_start: &mut USER_P_START,
        }
    }
}

pub fn bit_map() -> &'static mut [u8] {
    unsafe {
        &mut BIT_MAP_DATA
    }
}

pub fn init() {
    let total_mem = asm::memory_size();
    let free_mem = total_mem as usize - USED_MEM;
    let kernel_pages = KERNEL_MEM / PAGE_SIZE;
    let user_pages = (free_mem - KERNEL_MEM) / PAGE_SIZE;

    unsafe {
        KERNEL_P_START = USED_MEM;
        let m = bit_map();
        KERNEL_BIT_MAP = &mut m[..kernel_pages / 8];
    }
    unsafe {
        USER_P_START = USED_MEM + KERNEL_MEM;
        let m = bit_map();
        USER_BIT_MAP = &mut m[kernel_pages / 8..kernel_pages / 8 + user_pages / 8];
    }
}