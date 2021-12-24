use rlib::bitmap::Bitmap;

use crate::{asm, println};

const USED_MEM: usize = 8 * 1024 * 1024;
const KERNEL_MEM: usize = 4 * 1024 * 1024;
const PAGE_SIZE: usize = 4 * 1024;

const PAGE_POOL_ST_SZ: usize = core::mem::size_of::<PagePool>();

/// 128kb bit map
const BIT_MAP_SIZE: usize = (4 * 1024 * 1024 * 1024 / PAGE_SIZE as u64 / 8) as usize;

static mut BIT_MAP_DATA: [u8; BIT_MAP_SIZE] = [0u8; BIT_MAP_SIZE];
static mut PAGE_POOL_ST: [u8; PAGE_POOL_ST_SZ * 2] = [0u8; PAGE_POOL_ST_SZ * 2];

pub struct PagePool {
    pub bitmap: &'static mut [u8],
    pub pool_sz: usize,
    pub p_start: usize,
}

pub fn kernel_pool() -> &'static mut PagePool {
    unsafe {
        let p: &mut [PagePool; 2] = core::mem::transmute(&mut BIT_MAP_DATA);
        &mut p[0]
    }
}

pub fn user_pool() -> &'static mut PagePool {
    unsafe {
        let p: &mut [PagePool; 2] = core::mem::transmute(&mut BIT_MAP_DATA);
        &mut p[1]
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

    let k = kernel_pool();
    let u = user_pool();

    let m = bit_map();
    k.p_start = USED_MEM;
    k.bitmap = &mut m[..kernel_pages / 8];
    k.bitmap.init();
    k.pool_sz = KERNEL_MEM;

    let m = bit_map();
    u.p_start = USED_MEM + KERNEL_MEM;
    u.bitmap = &mut m[kernel_pages / 8..kernel_pages / 8 + user_pages / 8];
    u.bitmap.init();
    u.pool_sz = user_pages * PAGE_SIZE;
}