use rlib::bitmap::Bitmap;

use crate::err::SE;
use crate::mem::{fill_zero, k_lock, kernel_pool, PAGE_SIZE, PagePool, u_lock, user_pool, v_pool, VPool};
use crate::mem::page::{DEFAULT_PT_ATTR, map_page, page_dir, PDE_START, VirtualAddress};
use crate::println;
use crate::thread::current_pcb;

pub trait VAlloc {
    /// try to alloc continuous pages in virtual memory space
    fn v_alloc(&mut self, pages: usize) -> Result<usize, SE>;
}

pub trait PAlloc {
    /// try to alloc one page in physical memory space, not required to be continuous
    fn p_alloc(&mut self) -> Result<usize, SE>;
}

impl VAlloc for VPool {
    fn v_alloc(&mut self, pages: usize) -> Result<usize, SE> {
        let v = v_pool();

        let bit_i = v.bitmap.try_alloc(pages);
        if bit_i < 0 {
            return Err("memory overflow");
        }

        v.bitmap.fill_n(bit_i as usize, pages);

        Ok(
            v.v_start + (bit_i as usize) * PAGE_SIZE
        )
    }
}

impl PAlloc for PagePool {
    fn p_alloc(&mut self) -> Result<usize, SE> {
        self.avl_pages -= 1;
        let bit_i = self.bitmap.try_alloc(1);
        if bit_i < 0 { return Err("memory overflow"); }
        self.bitmap.set(bit_i as usize, true);

        let p = self.p_start + (bit_i as usize) * PAGE_SIZE;
        // return physical address of this page
        Ok(
            p
        )
    }
}

#[derive(PartialEq, Debug)]
pub enum Pool {
    KERNEL,
    USER,
}

pub fn v2p(p: Pool, v: usize) -> usize {
    assert_eq!(p, Pool::KERNEL, "kernel");
    let pd = page_dir(PDE_START);
    let pt = pd[v.pde_i()].sub_table();
    (pt[v.pte_i()].data & 0xfffff000) | (v & 0xfff)
}

// allocate only one page by virtual address
pub fn alloc_one(p: Pool, v_ad: usize, init: bool) -> Result<usize, SE> {
    use rlib::bitmap::Bitmap;
    assert_eq!(v_ad % PAGE_SIZE, 0, "virtual address 0x{:08X} % {} != 0", v_ad, PAGE_SIZE);
    let lk = if p == Pool::KERNEL { k_lock() } else { u_lock() };

    let _gd = lk.map(|x| x.lock());

    let pcb = current_pcb();
    let pd = if p == Pool::KERNEL { PDE_START } else { pcb.pd };

    let v = if p == Pool::KERNEL { v_pool() } else {
        pcb.v_pool()
    };

    let bit_i = (v_ad - v.v_start) / PAGE_SIZE;
    assert!(!v.bitmap.test(bit_i), "0x{:08X} is allocated", v_ad);


    let pp = if p == Pool::KERNEL { kernel_pool() } else { user_pool() };
    println!("1");
    v.bitmap.set(bit_i, true);
    println!("2");
    let p = pp.p_alloc()?;
    println!("3");

    println!("p_alloc success p = 0x{:08X}", p);
    map_page(pd, v_ad, p, DEFAULT_PT_ATTR, false, true)?;

    if init {
        fill_zero(v_ad, PAGE_SIZE);
    }
    Ok(v_ad)
}

pub fn pg_alloc(p: Pool, pages: usize, init: bool) -> Result<usize, SE> {
    let lk = if p == Pool::KERNEL { k_lock() } else { u_lock() };
    let _gd = lk.map(|x| x.lock());

    let pcb = current_pcb();
    let pd = if p == Pool::KERNEL { PDE_START } else { pcb.pd };

    let v = if p == Pool::KERNEL { v_pool() } else {
        pcb.v_pool()
    };

    let pp = if p == Pool::KERNEL { kernel_pool() } else { user_pool() };
    if pp.avl_pages < pages {
        return Err("memory not enough");
    }

    // virtual memory is required to be continuous
    let v_start = v.v_alloc(pages)?;

    // physical memory is not required to be continuous
    // we should mapping between virtual address to physical page by page
    for i in 0..pages {
        let p_a = pp.p_alloc()?;
        map_page(pd, v_start + i * PAGE_SIZE, p_a, DEFAULT_PT_ATTR, false, true)?;
    }

    if init {
        fill_zero(v_start, PAGE_SIZE * pages);
    }
    Ok(v_start)
}
