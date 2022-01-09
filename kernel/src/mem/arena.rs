use rlib::{alloc_static, div_up, size_of};
use rlib::link::{LinkedList, Node};

use crate::{c_println, Pool, println};
use crate::mem::{fill_zero, k_lock, PAGE_SIZE, pg_alloc, u_lock, v_pool};
use crate::mem::alloc::VAlloc;
use crate::thread::current_pcb;

pub const DESC_CNT: usize = 7;
pub const MAX_BLK_SIZE: usize = 1024;

alloc_static!(K_DESCS, k_descs, [BlkDesc; DESC_CNT]);

/// arena is head of a page(4K)
#[repr(C)]
pub struct Arena {
    // block size of this page
    desc: usize,
    count: usize,
    large: bool,
}

impl Arena {
    pub fn desc(&self) -> Option<&'static mut BlkDesc> {
        if self.desc == 0 { None } else { Some(cst!(self.desc)) }
    }

    pub fn block(&self, i: usize) -> usize {
        let p = self as *const _ as usize;
        p + size_of!(Arena) + self.desc().as_ref().unwrap().blk_sz * i
    }
}

pub const PADDING: usize = 32;


/// for large arena, block pointer = page off + size_of::<Arena>
/// for other arena, block pointer = page off + size_of::<Arena> + i * block size
/// for all blocks, page off = block pointer & 0xfffff000
#[repr(transparent)]
pub struct Blk {
    pointers: [usize; 2],
}

impl Blk {
    pub fn arena(&self) -> &'static mut Arena {
        cst!((unsafe { self as *const _ as usize }) & 0xfffff000)
    }
}

impl Node for Blk {
    fn pointers_mut(&mut self) -> &mut [usize] {
        &mut self.pointers
    }

    fn pointers(&self) -> &[usize] {
        &self.pointers
    }
}

#[repr(C)]
pub struct BlkDesc {
    pub blk_sz: usize,
    pub blocks: usize,
    pub frees: LinkedList<Blk, 32>,
}

impl BlkDesc {
    pub fn init(&mut self) {
        self.frees.init(0, 1);
    }
}

pub fn init_descs(d: &mut [BlkDesc]) {
    let mut block_size: usize = 16;
    for i in 0..DESC_CNT {
        d[i].blk_sz = block_size;
        d[i].blocks = (PAGE_SIZE - size_of!(Arena)) / block_size;
        d[i].init();
        block_size *= 2;
    }
}

pub fn init() {
    init_descs(k_descs());
}

// malloc memory in kernel space
pub fn malloc(size: usize) -> usize {
    let cur = current_pcb();

    let l = if cur.user() { u_lock() } else { k_lock() };
    let pool = if cur.user() { Pool::USER } else { Pool::KERNEL };
    let ds = if cur.user() { &mut cur.desc } else { k_descs() };
    let _gd = l.map(|x| x.lock());

    // allocate page by page if size > 1024
    if size > MAX_BLK_SIZE {
        let pages = div_up!(size + size_of!(Arena), PAGE_SIZE);
        let p = pg_alloc(pool, pages, true).unwrap();
        let a: &'static mut Arena = cst!(p);
        a.desc = 0;
        a.count = pages;
        a.large = true;
        return p + size_of!(Arena);
    }

    let i = (0..DESC_CNT).find(|x| ds[*x].blk_sz >= size).unwrap();

    // initialize blocks
    // create area, link them
    if ds[i].frees.is_empty() {
        let p = pg_alloc(pool, 1, true).unwrap();
        let a: &'static mut Arena = cst!(p);
        a.desc = unsafe { ds.as_ptr().add(i) } as usize;
        a.large = false;
        a.count = ds[i].blocks;


        for j in 0..ds[i].blocks {
            let b: &'static mut Blk = cst!(a.block(j));
            ds[i].frees.append(b);
        }
    }

    let b = ds[i].frees.pop_head().unwrap();
    fill_zero(b as *const _ as usize, ds[i].blk_sz);
    let a = b.arena();
    a.count -= 1;
    b as *const _ as usize
}

pub fn free(p: usize) {
    let cur = current_pcb();
    let lk = if cur.user() { u_lock() } else { k_lock() };
    let _gd = lk.map(|x| x.lock());

    let b: &'static mut Blk = cst!(p);
    b.pointers.fill(0);
    let a = b.arena();

    let v_p = if cur.user() { cur.v_pool() } else { v_pool() };

    if a.large {
        v_p.free(a as *const _ as usize, a.count);
        return;
    }

    // collect free block
    let d = a.desc().unwrap();
    d.frees.append(b);
    a.count += 1;

    if a.count == d.blocks {
        // clear the list
        d.init();
        v_p.free(a as *const _ as usize, 1);
    }
}