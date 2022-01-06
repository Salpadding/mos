use crate::asm::{SELECTOR_U_CODE, SELECTOR_U_DATA};
use crate::int::{disable_int, set_int};
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc};
use crate::Pool;
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
    ctx.esp = pg_alloc(Pool::KERNEL, USER_PAGES).unwrap() as u32;
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
    let pcb_off = pg_alloc(Pool::KERNEL, 1).unwrap();
    let pcb = PCB::new(name, priority, pcb_off);
    pcb.init(entry, rt, args);
    // mark as user process
    // pcb.pd = pg_alloc(Pool::KERNEL, 1).unwrap();

    let old = disable_int();
    ready().append(pcb);
    all().append(pcb);
    set_int(old);
}