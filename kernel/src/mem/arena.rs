use rlib::{alloc_static, div_up, size_of};
use rlib::link::{LinkedList, Node};

use crate::{c_println, Pool, println};
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc};

pub const DESC_CNT: usize = 7;
pub const MAX_BLK_SIZE: usize = 1024;

alloc_static!(K_DESCS, k_descs, [BlkDesc; DESC_CNT]);

#[repr(C)]
pub struct Arena {
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
    blk_sz: usize,
    blocks: usize,
    frees: LinkedList<Blk>,
    // paddings for initialize linked list
    padding: [u8; PADDING],
}

impl BlkDesc {
    fn init(&mut self) {
        let hd = unsafe { self.padding.as_ptr() as usize };
        let tail = hd + PADDING / 2;
        self.frees.init(0, 1, cst!(hd), cst!(tail));
    }
}


pub fn init() {
    let mut block_size: usize = 16;
    for i in 0..DESC_CNT {
        k_descs()[i].blk_sz = block_size;
        k_descs()[i].blocks = (PAGE_SIZE - size_of!(Arena)) / block_size;
        k_descs()[i].init();
        block_size *= 2;
    }

    // print blocks

    for i in 0..DESC_CNT {
        let b = unsafe { k_descs().as_ptr().add(i) };
        let bm = &k_descs()[i];
        c_println!("block {} at 0x{:08X}", i, b as usize);
        c_println!("block {} size = {}", i, bm.blk_sz);
        c_println!("block {}'s padding at 0x{:08X}", i, bm.padding.as_ptr() as usize);
        c_println!("block {}'s head at 0x{:08X}", i, bm.frees.head);
        c_println!("block {}'s tail at 0x{:08X}", i, bm.frees.tail);
        c_println!("list len = {}", bm.frees.len());
    }
}

// malloc memory in kernel space
pub fn malloc(size: usize) -> usize {
    // allocate page by page if size > 1024
    if size > MAX_BLK_SIZE {
        let pages = div_up!(size + size_of!(Arena), PAGE_SIZE);
        let p = pg_alloc(Pool::KERNEL, pages, true).unwrap();
        let a: &'static mut Arena = cst!(p);
        a.desc = 0;
        a.count = pages;
        a.large = true;
        return p + size_of!(Arena);
    }

    let k = k_descs();
    let i = (0..DESC_CNT).find(|x| k[*x].blk_sz >= size).unwrap();

    // initialize blocks
    // create area, link them
    if k[i].frees.is_empty() {
        let p = pg_alloc(Pool::KERNEL, 1, true).unwrap();
        let a: &'static mut Arena = cst!(p);
        a.desc = unsafe { k.as_ptr().add(i) } as usize;
        a.large = false;
        a.count = k[i].blocks;

        for i in 0..k[i].blocks {
            let b: &'static mut Blk = cst!(a.block(i));
            k[i].frees.append(b);
        }
    }

    let b = k[i].frees.pop_head().unwrap();
    fill_zero(b as *const _ as usize, k[i].blk_sz);
    let a = b.arena();
    a.count -= 1;
    0
}