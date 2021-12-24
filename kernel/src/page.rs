use crate::println;

const PDE_START: usize = 0x100000;
const PE_SIZE: usize = 4;
const PT_LEN: usize = 1024;
const PT_SIZE: usize = PE_SIZE * PT_LEN;
const OS_MEM_OFF: usize = 0xc0000000;

// 1m area for page
const PAGE_AREA_SIZE: usize = 1024 * 1024;

pub type PageTable = &'static mut [PageTableEntry];

#[repr(transparent)]
pub struct PageTableEntry {
    data: usize,
}

impl PageTableEntry {
    fn zero() -> Self {
        PageTableEntry { data: 0 }
    }

    pub fn new(p_start: usize) -> PageTableEntry {
        Self {
            data: p_start | 7
        }
    }
}

pub fn page_directory() -> PageTable {
    &mut page_area()[..PT_LEN]
}

pub fn page_area() -> &'static mut [PageTableEntry] {
    let p = PDE_START;
    let sz = PAGE_AREA_SIZE / PE_SIZE;
    unsafe {
        core::slice::from_raw_parts_mut(p as *mut _, sz)
    }
}


pub fn page_tables() -> PageTable {
    &mut page_area()[PT_LEN..]
}

fn init_pde() {
    let p = page_area();
    for t in p.iter_mut() {
        *t = PageTableEntry::zero();
    }
}

fn new_pte(pte_i: usize, p_start: usize, size: usize) -> usize {
    // get pde of v_start
    let pt = page_tables();
    for i in 0..(size >> 12) {
        pt[pte_i + i] = PageTableEntry::new(p_start + (i << 12));
    }
    size >> 12
}


// pte_cnt must be 1K aligned
fn map_pd(start_pde: usize, mut start_pte: usize, pte_cnt: usize) {
    let pd = page_directory();
    for i in 0..(pte_cnt >> 10) {
        pd[start_pde + i] = PageTableEntry::new(
            PDE_START + PT_SIZE + PE_SIZE * start_pte,
        );
        start_pte += 1024;
    }
}

pub fn init_page() -> usize {
    init_pde();

    let pts = new_pte(0, 0, 8 * 1024 * 1024);
    map_pd(0, 0, pts);
    map_pd(OS_MEM_OFF >> 22, 0, pts);


    // allocate virtual memory for stack (2M)
    let stack_size = 2 * 1024 * 1024;
    new_pte(pts, 0x600000, stack_size);

    let pd = page_directory();
    pd[(OS_MEM_OFF >> 22) - 1] = PageTableEntry::new(
        PDE_START + PT_SIZE + pts * PE_SIZE
    );

    let stack_high = (((OS_MEM_OFF >> 22) - 1) << 22) + stack_size - 0x10;
    stack_high
}

