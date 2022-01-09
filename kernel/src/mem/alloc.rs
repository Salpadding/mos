use rlib::bitmap::Bitmap;

use crate::{OS_MEM_OFF, println};
use crate::err::SE;
use crate::mem::{
    fill_zero, k_lock, kernel_pool, PAGE_SIZE, PagePool, u_lock, user_pool, v_pool, VPool,
};
use crate::mem::page::{DEFAULT_PT_ATTR, LOOP_BACK_PD, map_page, page_table, PDE_START, RESERVED_MEM, USER_P_START, USER_V_START, VirtualAddress};
use crate::thread::current_pcb;

pub trait VAlloc {
    /// try to alloc continuous pages in virtual memory space
    fn v_alloc(&mut self, pages: usize) -> Result<usize, SE>;

    /// free pages, also free bitmap in physical pool
    fn free(&mut self, off: usize, pages: usize);


    /// remove bitmap only
    fn remove(&mut self, off: usize, pages: usize);
}

pub trait PAlloc {
    /// try to alloc one page in physical memory space, not required to be continuous
    fn p_alloc(&mut self) -> Result<usize, SE>;

    fn remove(&mut self, off: usize);
}


impl VAlloc for VPool {
    fn v_alloc(&mut self, pages: usize) -> Result<usize, SE> {
        let v = self;

        let bit_i = v.bitmap.try_alloc(pages);
        if bit_i < 0 {
            return Err("memory overflow");
        }

        v.bitmap.fill_n(bit_i as usize, pages, true);

        Ok(v.v_start + (bit_i as usize) * PAGE_SIZE)
    }

    fn free(&mut self, off: usize, pages: usize) {
        let p = v2p(off);
        let phy = if p >= USER_P_START { user_pool() }  else { kernel_pool() };

        for i in 0..pages {
            let v = off + i * PAGE_SIZE;
            let p = v2p(v);
            phy.remove(p);

            // remove pte
            let pte: *mut u32 = pte_ptr(v) as *mut _;
            let p = pte as usize;

            // flush page table
            unsafe {
                *pte = *pte & !1;
                asm!("invlpg [{}]", in(reg) p);
            }

        }

        self.remove(off, pages);
    }

    fn remove(&mut self, off: usize, pages: usize) {
        let start_i = (off - self.v_start) / PAGE_SIZE;
        self.bitmap.fill_n(start_i, pages, false);
    }
}

impl PAlloc for PagePool {
    fn p_alloc(&mut self) -> Result<usize, SE> {
        self.avl_pages -= 1;
        let bit_i = self.bitmap.try_alloc(1);
        if bit_i < 0 {
            return Err("memory overflow");
        }
        self.bitmap.set(bit_i as usize, true);

        let p = self.p_start + (bit_i as usize) * PAGE_SIZE;
        // return physical address of this page
        Ok(p)
    }

    fn remove(&mut self, off: usize) {
        self.bitmap.set((off - self.p_start) / PAGE_SIZE, false);
    }
}

#[derive(PartialEq, Debug)]
pub enum Pool {
    KERNEL,
    USER,
}

pub fn pte_ptr(v: usize) -> *const u8 {
    // get address of page table entry by loopback
    let x: usize = 0xffc00000 | v.pde_i() << 12 | v.pte_i() * 4;
    x as *const _
}

pub fn v2p(v: usize) -> usize {
    // get address of page table entry by loopback
    let x: usize = 0xffc00000 | v.pde_i() << 12 | v.pte_i() * 4;
    let y: *const usize = x as *const _;
    // physical address of page table
    unsafe { *y & 0xfffff000 | v & 0xfff }
}

// allocate only one page by virtual address
pub fn alloc_one(p: Pool, v_ad: usize, init: bool) -> Result<usize, SE> {
    assert_eq!(
        v_ad % PAGE_SIZE,
        0,
        "virtual address 0x{:08X} % {} != 0",
        v_ad,
        PAGE_SIZE
    );
    let lk = if p == Pool::KERNEL {
        k_lock()
    } else {
        u_lock()
    };

    let _gd = lk.map(|x| x.lock());

    let pcb = current_pcb();
    let pd = if p == Pool::KERNEL { PDE_START } else { pcb.pd };

    let v = if p == Pool::KERNEL {
        v_pool()
    } else {
        pcb.v_pool()
    };

    let bit_i = (v_ad - v.v_start) / PAGE_SIZE;
    assert!(!v.bitmap.test(bit_i), "0x{:08X} is allocated", v_ad);

    let pp = if p == Pool::KERNEL {
        kernel_pool()
    } else {
        user_pool()
    };
    v.bitmap.set(bit_i, true);
    let p = pp.p_alloc()?;

    map_page(pd, v_ad, p, DEFAULT_PT_ATTR, false, true)?;

    if init {
        fill_zero(v_ad, PAGE_SIZE);
    }
    Ok(v_ad)
}

pub fn pg_alloc(p: Pool, pages: usize, init: bool) -> Result<usize, SE> {
    let lk = if p == Pool::KERNEL {
        k_lock()
    } else {
        u_lock()
    };
    let _gd = lk.map(|x| x.lock());

    let pcb = current_pcb();
    let pd = if p == Pool::KERNEL { PDE_START } else { pcb.pd };

    let v = if p == Pool::KERNEL {
        v_pool()
    } else {
        pcb.v_pool()
    };

    let pp = if p == Pool::KERNEL {
        kernel_pool()
    } else {
        user_pool()
    };
    if pp.avl_pages < pages {
        return Err("memory not enough");
    }

    // virtual memory is required to be continuous
    let v_start = v.v_alloc(pages)?;

    // physical memory is not required to be continuous
    // we should mapping between virtual address to physical page by page
    for i in 0..pages {
        let p_a = pp.p_alloc()?;
        map_page(
            pd,
            v_start + i * PAGE_SIZE,
            p_a,
            DEFAULT_PT_ATTR,
            false,
            true,
        )?;
    }

    if init {
        fill_zero(v_start, PAGE_SIZE * pages);
    }
    Ok(v_start)
}
