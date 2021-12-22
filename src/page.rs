use crate::println;

const PDE_START: usize = 0x100000;
const PE_SIZE: usize = 4;
const PT_LEN: usize = 1024;
const PT_SIZE: usize = PE_SIZE * PT_LEN;
const OS_MEM_OFF: usize = 0xc0000000;

pub type PageTable = [PageTableEntry; PT_LEN];

#[repr(transparent)]
pub struct PageTableEntry {
    data: usize,
}

impl PageTableEntry {
    fn zero() -> Self {
        PageTableEntry { data: 0 }
    }

    // get page tables for a page descriptor entry
    pub fn page_table(&mut self) -> &mut PageTable {
        unsafe { &mut *((self.data & 0xfffff000) as *mut PageTable) }
    }

    pub fn new(p_start: usize, attr: usize) -> PageTableEntry {
        Self {
            data: p_start | attr
        }
    }
}



pub fn page_directory() -> &'static mut PageTable {
    unsafe { &mut *(PDE_START as *mut PageTable) }
}

pub fn page_tables() -> &'static mut [PageTableEntry] {
    let p = PDE_START + PT_SIZE;
    let sz = PT_LEN * PT_LEN;
    unsafe {
        core::slice::from_raw_parts_mut(p as *mut _, sz)
    }
}

fn init_pde() {
    let p = page_directory();
    for i in 0..PT_LEN {
        p[i] = PageTableEntry::zero();
    }
}

fn new_pte(start_pte_i: usize, p_start: usize, size: usize) -> usize{
    // get pde of v_start
    let pt = page_tables();
    for i in 0..(size >> 12) {
        pt[start_pte_i + i] = PageTableEntry::new(p_start + (i << 12), 7);
    }
    size >> 12
}

fn map_pd(start_pde: usize, mut start_pte: usize, pte_cnt: usize) {
    let pd = page_directory();
    for i in 0..(pte_cnt >> 10) {
        pd[start_pde + i] = PageTableEntry::new(
            PDE_START + PT_SIZE + PE_SIZE * start_pte,
                    7
        );
        start_pte += 1024;
    }
}

pub fn init_page() {
    init_pde();

    let pts = new_pte(0, 0, 8 * 1024 * 1024);
    map_pd(0, 0, pts);
    map_pd(OS_MEM_OFF >> 22, 0, pts);
}

