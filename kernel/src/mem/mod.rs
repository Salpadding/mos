pub use {alloc::pg_alloc, alloc::Pool, page::init_page, page::page_enabled};
use rlib::bitmap::Bitmap;

use crate::{asm, println};
use crate::mem::page::{p_alloc, PDE_START, PT_SIZE, RESERVED_MEM};

mod page;
mod alloc;

const KERNEL_MEM: usize = 3 << 20;
pub const PAGE_SIZE: usize = 4 * 1024;
const BUF_ST_SIZE: usize = 128;

/// 128kb bit map
const BIT_MAP_SIZE: usize = (4 * 1024 * 1024 * 1024 / PAGE_SIZE as u64 / 8) as usize;
static mut BIT_MAP: usize = 0;

/// allocate buffer to global object
static mut BUF: usize = 0;

/// pool of virtual address
struct VPool {
    bitmap: &'static mut [u8],
    v_start: usize,
}

macro_rules! cast {
    ($t: ident, $off: expr) => {
       unsafe {
            let p: *mut $t = (BUF + $off) as *mut _;
            &mut *p
       }
    };
}

struct PagePool {
    bitmap: &'static mut [u8],
    total_pages: usize,
    avl_pages: usize,
    p_start: usize,
}

impl PagePool {
    fn size(&self) -> usize {
        self.total_pages * PAGE_SIZE
    }
}

fn kernel_pool() -> &'static mut PagePool {
    cast!(PagePool, 0)
}

fn user_pool() -> &'static mut PagePool {
    cast!(PagePool, BUF_ST_SIZE)
}

fn v_pool() -> &'static mut VPool {
    cast!(VPool, BUF_ST_SIZE * 2)
}

fn bit_map() -> &'static mut [u8] {
    unsafe {
        if BIT_MAP == 0 {
            BIT_MAP = p_alloc(BIT_MAP_SIZE / PAGE_SIZE, true).unwrap();
        }
        core::slice::from_raw_parts_mut(
            BIT_MAP as *mut _,
            BIT_MAP_SIZE,
        )
    }
}

pub fn fill_zero(start: usize, len: usize) {
    let p = start as *mut usize;
    unsafe {
        for i in 0..len / core::mem::size_of::<usize>() {
            *p.add(i) = 0;
        }
    }
}

pub fn debug() {
    let k = crate::mem::kernel_pool();
    let u = crate::mem::user_pool();

    println!("kernel: pool size = {}M, p_start = {}M bitmap len = {}", k.size() / 1024 / 1024, k.p_start / 1024 / 1024, k.bitmap.len());
    println!("user  : pool size = {}M, p_start = {}M bitmap len = {}", u.size() / 1024 / 1024, u.p_start / 1024 / 1024, u.bitmap.len());
}

pub fn init() {
    unsafe { println!("bitmap address = 0x{:08X}", BIT_MAP); }
    unsafe {
        BUF = p_alloc(1, true).unwrap();
    }
    // initialize kernel area and bit map
    fill_zero(RESERVED_MEM, KERNEL_MEM);

    let total_mem = asm::memory_size() / PAGE_SIZE as u32 * PAGE_SIZE as u32;
    let user_mem = total_mem as usize - RESERVED_MEM - KERNEL_MEM;
    let kernel_pages = KERNEL_MEM / PAGE_SIZE;
    let user_pages = user_mem / PAGE_SIZE;

    let k = kernel_pool();
    let u = user_pool();
    let v = v_pool();

    let m = bit_map();

    k.p_start = RESERVED_MEM;
    k.bitmap = &mut m[..kernel_pages / 8];
    k.total_pages = kernel_pages;
    k.avl_pages = k.total_pages;

    let m = bit_map();
    u.p_start = RESERVED_MEM + KERNEL_MEM;
    u.bitmap = &mut m[kernel_pages / 8..kernel_pages / 8 + user_pages / 8];
    u.total_pages = user_pages;
    u.avl_pages = u.total_pages;

    let m = bit_map();
    v.bitmap = &mut m[k.bitmap.len() + u.bitmap.len()..(k.bitmap.len() + u.bitmap.len() + k.bitmap.len())];
    v.v_start = page::OS_MEM_OFF + RESERVED_MEM;
}
