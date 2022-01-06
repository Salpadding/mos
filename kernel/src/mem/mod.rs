use rlib::bitmap::Bitmap;
pub use {alloc::pg_alloc, alloc::Pool, page::init_page, page::page_enabled, page::PageTable, page::PT_LEN};

use crate::mem::page::{static_alloc, PDE_START, PT_SIZE, RESERVED_MEM};
use crate::{asm, println};
use crate::thread::sync::Lock;

mod alloc;
mod page;

pub static mut K_LOCK: [u8; 256] = [0u8; 256];
pub static mut K_LOCK_REF: usize = 0;

pub static mut U_LOCK: [u8; 256] = [0u8; 256];
pub static mut U_LOCK_REF: usize = 0;


pub fn k_lock() -> Option<&'static mut Lock> {
    if unsafe { K_LOCK_REF == 0 } {
        None
    } else {
        Some(cst!(K_LOCK_REF))
    }
}

pub fn u_lock() -> Option<&'static mut Lock> {
    if unsafe { U_LOCK_REF == 0 } {
        None
    } else {
        Some(cst!(U_LOCK_REF))
    }
}

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

// kernel physical memory pool
fn kernel_pool() -> &'static mut PagePool {
    cast!(PagePool, 0)
}


// user physical memory pool
fn user_pool() -> &'static mut PagePool {
    cast!(PagePool, BUF_ST_SIZE)
}

// kernel virtual memory pool
fn v_pool() -> &'static mut VPool {
    cast!(VPool, BUF_ST_SIZE * 2)
}

fn bit_map() -> &'static mut [u8] {
    unsafe {
        if BIT_MAP == 0 {
            BIT_MAP = static_alloc(BIT_MAP_SIZE / PAGE_SIZE, true).unwrap();
        }
        core::slice::from_raw_parts_mut(BIT_MAP as *mut _, BIT_MAP_SIZE)
    }
}

static mut BIT_MAP_USES: usize = 0;

fn alloc_bit_map(len: usize) -> &'static mut [u8] {
    let bits = bit_map();

    let r = unsafe { &mut bits[BIT_MAP_USES..BIT_MAP_USES + len] };
    unsafe {
        BIT_MAP_USES += len;
    }
    r
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

    println!(
        "kernel: pool size = {}M, p_start = {}M bitmap len = {}",
        k.size() / 1024 / 1024,
        k.p_start / 1024 / 1024,
        k.bitmap.len()
    );
    println!(
        "user  : pool size = {}M, p_start = {}M bitmap len = {}",
        u.size() / 1024 / 1024,
        u.p_start / 1024 / 1024,
        u.bitmap.len()
    );
}

pub fn init() {
    // initialize bitmap
    unsafe {
        BUF = static_alloc(1, true).unwrap();
    }

    assert!(
        core::mem::size_of::<PagePool>() < BUF_ST_SIZE,
        "size of page pool"
    );
    assert!(
        core::mem::size_of::<VPool>() < BUF_ST_SIZE,
        "size of v pool"
    );

    // initialize kernel area and bit map
    fill_zero(RESERVED_MEM, KERNEL_MEM);

    let total_mem = asm::memory_size() / PAGE_SIZE as u32 * PAGE_SIZE as u32;
    let user_mem = total_mem as usize - RESERVED_MEM - KERNEL_MEM;
    let kernel_pages = KERNEL_MEM / PAGE_SIZE;
    let user_pages = user_mem / PAGE_SIZE;

    let k = kernel_pool();
    let u = user_pool();
    let v = v_pool();

    k.p_start = RESERVED_MEM;
    k.bitmap = alloc_bit_map(kernel_pages / 8);

    k.total_pages = kernel_pages;
    k.avl_pages = k.total_pages;

    u.p_start = RESERVED_MEM + KERNEL_MEM;
    u.bitmap = alloc_bit_map(user_pages / 8);
    u.total_pages = user_pages;
    u.avl_pages = u.total_pages;

    v.bitmap = alloc_bit_map(kernel_pages / 8);
    v.v_start = page::OS_MEM_OFF + RESERVED_MEM;
}
