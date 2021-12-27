use rlib::bitmap::Bitmap;

use crate::err::SE;
use crate::mem::{fill_zero, kernel_pool, PAGE_SIZE, PagePool, user_pool, v_pool, VPool};
use crate::mem::page::map_page;
use crate::println;

pub trait VAlloc {
    /// try to alloc continuous pages in virtual memory space
    fn v_alloc(&mut self, pages: usize) -> Result<usize, SE>;
}

pub trait PAlloc {
    /// try to alloc one page in physical memory space, not required to be continuous
    fn p_alloc(&mut self, init: bool) -> Result<usize, SE>;
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
    fn p_alloc(&mut self, init: bool) -> Result<usize, SE> {
        self.avl_pages -= 1;
        let bit_i = self.bitmap.try_alloc(1);
        if bit_i < 0 { return Err("memory overflow"); }
        self.bitmap.set(bit_i as usize, true);

        let p = self.p_start + (bit_i as usize) * PAGE_SIZE;
        if init {
            fill_zero(p, PAGE_SIZE);
        }
        // return physical address of this page
        Ok(
            p
        )
    }
}

#[derive(PartialEq)]
pub enum Pool {
    KERNEL,
    USER,
}


pub fn pg_alloc(p: Pool, pages: usize) -> Result<usize, SE> {
    if p != Pool::KERNEL {
        return Err("not implemented");
    }

    let v = v_pool();
    let pp = if p == Pool::KERNEL { kernel_pool() } else { user_pool() };
    if pp.avl_pages < pages {
        return Err("memory not enough");
    }

    // virtual memory is required to be continuous
    let v_start = v.v_alloc(pages)?;

    // physical memory is not required to be continuous
    // we should mapping between virtual address to physical page by page
    for i in 0..pages {
        let p_a = pp.p_alloc(false)?;
        map_page(v_start + i * PAGE_SIZE, p_a, 7, false, true)?;
    }

    Ok(v_start)
}
