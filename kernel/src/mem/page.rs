use crate::asm::KERNEL_ENTRY;
use crate::err::SE;
use crate::mem::{fill_zero, KERNEL_MEM, kernel_pool, PAGE_SIZE};
use crate::mem::alloc::PAlloc;
use crate::{println, c_println};
use crate::thread::{MAIN_PRIORITY, PCB, PCB_PAGES, PCB_SIZE, Routine, Status};

pub const PE_SIZE: usize = 4;
pub const PT_LEN: usize = 1024;
pub const PT_SIZE: usize = PE_SIZE * PT_LEN;
pub const OS_MEM_OFF: usize = 0xc0000000;
pub const RESERVED_MEM: usize = 5 << 20;
pub const USER_P_START: usize = 8 << 20;
pub const USER_V_START: usize = 8 << 20;
pub const DEFAULT_PT_ATTR: u16 = 7;

// 1m area for page
pub const PAGE_AREA_SIZE: usize = 1024 * 1024;

// page directory must align to 4K
pub const PDE_START: usize = 0x10000;

pub const BUF_UPPER_BOUND: usize = 0x80000;

// for static alloc before page setup
static mut PD_USED: usize = 0;

pub const LOOP_BACK_PD: usize = 0xfffff000;


pub fn page_table(off: usize) -> PageTable {
    unsafe { core::slice::from_raw_parts_mut(off as *mut _, PT_LEN) }
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

pub type PageTable = &'static mut [PageTableEntry];

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    pub data: usize,
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

// allocate pages before setup page
pub fn static_alloc(pages: usize, init: bool) -> Result<usize, SE> {
    let off = unsafe { PDE_START + PT_SIZE + PD_USED * PT_SIZE };
    let avl = (BUF_UPPER_BOUND - off) / PAGE_SIZE;
    if avl < pages {
        return Err("overflow");
    }
    unsafe {
        PD_USED += pages;
    }

    if init {
        fill_zero(off, pages * PAGE_SIZE);
    }
    // avoid overflow
    Ok(off)
}

// map
pub fn map_page(pd: usize, v: usize, p: usize, flags: u16, trace: bool, alloc: bool) -> Result<(), SE> {
    let pd = page_table(pd);
    let pde_i = v.pde_i();

    if trace {
        println!("map 0x{:08X} to 0x{:08X}", v, p);
        println!("pd {} exists = {}", pde_i, pd[pde_i].exists());
    }

    if !pd[pde_i].exists() {
        let k = kernel_pool();
        let buf = if alloc {
            k.p_alloc()?
        } else {
            static_alloc(1, true)?
        };
        if trace {
            println!("create buf 0x{:08X}", buf);
        }
        pd[pde_i] = PageTableEntry::new(buf, flags);

        if alloc {
            fill_zero((PT_LEN - 1) << 22 | pde_i << 12, PAGE_SIZE);
        }
    }

    let pt = if alloc {
        // access physical memory by loopback
        page_table((PT_LEN - 1) << 22 | pde_i << 12)
    } else { pd[pde_i].sub_table() };

    if trace {
        println!("pte i = {}", v.pte_i());
        println!(
            "sub table physical address = :{:08X}",
            pd[pde_i].sub_table().as_ptr() as usize
        );
    }
    if pt[v.pte_i()].exists() {
        println!(
            "page table already exists v = 0x{:08X} p = 0x{:08X} pde_i = {}, pte_i = {}",
            v,
            p,
            pde_i,
            v.pte_i()
        );
    }

    pt[v.pte_i()] = PageTableEntry::new(p, flags);
    Ok(())
}

static mut PAGE_ENABLED: bool = false;

pub fn page_enabled() -> &'static mut bool {
    unsafe { &mut PAGE_ENABLED }
}

pub fn init_page() {
    // init bitmaps
    crate::mem::init();

    fill_zero(PDE_START, PT_SIZE);

    for i in 0..(RESERVED_MEM + KERNEL_MEM) / PAGE_SIZE {
        map_page(PDE_START, i * PAGE_SIZE, i * PAGE_SIZE, DEFAULT_PT_ATTR, false, false).unwrap();
    }
    for i in 0..RESERVED_MEM / PAGE_SIZE {
        map_page(PDE_START, OS_MEM_OFF + i * PAGE_SIZE, i * PAGE_SIZE, DEFAULT_PT_ATTR, false, false).unwrap();
    }

    // loopback page directory
    let pd = page_table(PDE_START);
    pd[PT_LEN - 1] = PageTableEntry::new(PDE_START, DEFAULT_PT_ATTR);

    let init_off = static_alloc(PCB_PAGES, true).unwrap();
    // init process
    // since we not paged memory, we cannot access 0xc0500000
    let init = PCB::new(
        "init",
        MAIN_PRIORITY,
        init_off,
    );

    // init thread is already running
    *init.status_mut() = Status::Running;
    let new_stack = OS_MEM_OFF + init.stack_off();

    println!("new stack = 0x{:08X}", new_stack);
    // println!("new stack");
    crate::asm::page_jmp(PDE_START, new_stack, KERNEL_ENTRY);
}
