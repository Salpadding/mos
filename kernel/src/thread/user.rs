use crate::asm::{SELECTOR_U_CODE, SELECTOR_U_DATA};
use crate::int::{disable_int, set_int};
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc, PT_LEN};
use crate::mem::page::{DEFAULT_PT_ATTR, OS_MEM_OFF, page_dir, PageTableEntry, PDE_START, USER_V_START};
use crate::{Pool, println, v2p};
use crate::mem::alloc::alloc_one;
use crate::thread::{current_pcb, PCB, Routine};
use crate::thread::data::{all, ready};
use crate::thread::reg::{IntCtx, KernelCtx};

const USER_PAGES: usize = 1;
const USER_E_FLAGS: u32 = (1 << 1) | (1 << 9);

pub extern "C" fn entry(rt: Routine, args: usize) {
    let cur = current_pcb();
    cur.stack += core::mem::size_of::<KernelCtx>();
    let ctx = cur.int_ctx();
    fill_zero(cur.stack, core::mem::size_of::<IntCtx>());
    ctx.ds = SELECTOR_U_DATA as u32;
    ctx.ss = ctx.ds;
    ctx.es = ctx.ss;
    ctx.eip = rt as usize as u32;
    ctx.cs = SELECTOR_U_CODE as u32;
    ctx.esp = alloc_one(Pool::USER, OS_MEM_OFF - (1 << 20), true).unwrap() as u32;
    ctx.e_flags = USER_E_FLAGS;
    ctx.esp += (PAGE_SIZE * USER_PAGES) as u32;
    ctx.esp -= 4;
    unsafe {
        let top: *mut u32 = ctx.esp as usize as *mut _;
        *top = args as u32;
    }
    ctx.esp -= 4;

    unsafe { asm!("mov esp, {0}", "jmp {1}", in(reg) cur.stack, in(reg) crate::asm::int_exit()); }
}

pub fn create(rt: Routine, args: usize, name: &str, priority: u8) {
    let pcb_off = pg_alloc(Pool::KERNEL, 1, true).unwrap();
    let pcb = PCB::new(name, priority, pcb_off);
    pcb.init(entry, rt, args);

    // initialize v start
    pcb.v_pool.v_start = USER_V_START;
    let bits_bytes = (OS_MEM_OFF - USER_V_START) / PAGE_SIZE / 8;
    let p = bits_bytes / PAGE_SIZE;
    let bit_map = pg_alloc(Pool::KERNEL, p, true).unwrap();
    pcb.v_pool.bitmap = unsafe {
        core::slice::from_raw_parts_mut(bit_map as *mut _, p * PAGE_SIZE)
    };


    // create page directory
    pcb.pd = v2p(Pool::KERNEL, pg_alloc(Pool::KERNEL, 1, true).unwrap());
    let mut pd = pcb.page_dir().unwrap();
    pd.copy_from_slice(page_dir(PDE_START));
    // loopback page table entry
    pd[PT_LEN - 1] = PageTableEntry::new(pcb.pd, DEFAULT_PT_ATTR);




    let old = disable_int();
    ready().append(pcb);
    all().append(pcb);
    set_int(old);
}