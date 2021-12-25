use crate::err::SE;
use crate::mem::{fill_zero, KERNEL_MEM, kernel_pool, PAGE_SIZE};
use crate::mem::alloc::PAlloc;
use crate::println;

pub const PE_SIZE: usize = 4;
pub const PT_LEN: usize = 1024;
pub const PT_SIZE: usize = PE_SIZE * PT_LEN;
pub const OS_MEM_OFF: usize = 0xc0000000;
pub const RESERVED_MEM: usize = 6 << 20;

// 1m area for page
pub const PAGE_AREA_SIZE: usize = 1024 * 1024;

// page directory must align to 4K
pub const PDE_START: usize = 0x40000;

pub const BUF_UPPER_BOUND: usize = 0x80000;

static mut PD_USED: usize = 0;


fn page_dir() -> PageTable {
    unsafe { core::slice::from_raw_parts_mut(PDE_START as *mut _, PT_LEN) }
}

pub trait VirtualAddress {
    fn pde_i(self) -> usize;
    fn pte_i(self) -> usize;
}

impl VirtualAddress for usize {
    #[inline]
    fn pde_i(self) -> usize {
        self >> 22
    }

    #[inline]
    fn pte_i(self) -> usize {
        (self >> 12) & 0x3ff
    }
}

type PageTable = &'static mut [PageTableEntry];

#[repr(transparent)]
struct PageTableEntry {
    data: usize,
}

impl PageTableEntry {
    fn zero() -> Self {
        PageTableEntry { data: 0 }
    }

    pub fn new(p_start: usize, flags: u16) -> PageTableEntry {
        Self {
            data: p_start | (flags as usize),
        }
    }

    pub fn exists(&self) -> bool {
        (self.data & 1) != 0
    }

    pub fn delete(&mut self) {
        self.data = self.data & (!1)
    }

    pub fn sub_table(&self) -> PageTable {
        let p = self.data & 0xfffff000;
        unsafe { core::slice::from_raw_parts_mut(p as *mut _, PT_LEN) }
    }
}

// map
pub fn map_page(v: usize, p: usize, flags: u16, trace: bool, alloc: bool) -> Result<(), SE> {
    let pd = page_dir();
    let pde_i = v.pde_i();

    if trace {
        println!("map 0x{:08X} to 0x{:08X}", v, p);
        println!("pd {} exists = {}", pde_i, pd[pde_i].exists());
    }
    if !pd[pde_i].exists() {
        let k = kernel_pool();
        let buf = if alloc { k.p_alloc(true)? } else {
            let off = unsafe { PDE_START + PT_SIZE + PD_USED * PT_SIZE };
            unsafe { PD_USED += 1; }
            // avoid overflow
            if off > BUF_UPPER_BOUND {
                return Err("overflow");
            }
            off
        };
        if trace {
            println!("create buf 0x{:08X}", buf);
        }
        pd[pde_i] = PageTableEntry::new(buf, flags);
    }

    let pt = pd[pde_i].sub_table();

    if trace {
        println!("pte i = {}", v.pte_i());
        println!("sub table physical address = :{:08X}", pd[pde_i].sub_table().as_ptr() as usize);
    }
    if pt[v.pte_i()].exists() {
        println!("page table already exists v = 0x{:08X} p = 0x{:08X} pde_i = {}, pte_i = {}", v, p, pde_i, v.pte_i());
    }
    pt[v.pte_i()] = PageTableEntry::new(p, flags);
    Ok(())
}

static mut PAGE_ENABLED: bool = false;

pub fn page_enabled() -> bool {
    unsafe { PAGE_ENABLED }
}

pub fn init_page() -> ! {
    unsafe { PAGE_ENABLED = true; };
    // init bitmaps
    crate::mem::init();

    fill_zero(PDE_START, PT_SIZE);

    for i in 0..(RESERVED_MEM + KERNEL_MEM) / PAGE_SIZE {
        map_page(i * PAGE_SIZE, i * PAGE_SIZE, 7, false, false).unwrap();
    }
    for i in 0..RESERVED_MEM / PAGE_SIZE {
        map_page(OS_MEM_OFF + i * PAGE_SIZE, i * PAGE_SIZE, 7, false, false).unwrap();
    }


    let new_stack = OS_MEM_OFF + (6 << 20) - 0x10;
    crate::asm::page_setup(PDE_START, new_stack);
}
