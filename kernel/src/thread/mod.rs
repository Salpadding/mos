use rlib::alloc_static;
use rlib::link::{LinkedList, Node};

use crate::{Pool, println};
use crate::asm::{IntCtx, reg_ctx, REG_CTX_LEN, switch_to};
use crate::err::SE;
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc};
use crate::thread::data::all;
use crate::thread::Status::{Ready, Running};

mod data;

pub type Routine = extern "C" fn();

pub const PCB_PAGES: usize = 1;
const STACK_MAGIC: u32 = 0x55aa55aa;
pub const PCB_SIZE: usize = PCB_PAGES * PAGE_SIZE;

static mut TICKS: u32 = 0;
// address of switch to
pub static mut SWITCH_TO: usize = 0;

pub fn ticks() -> &'static mut u32 {
    unsafe { &mut TICKS }
}

macro_rules! cur_pcb {
    () => {
        unsafe {
            let esp: u32;
            asm!("mov {}, esp", out(reg) esp);
            esp as usize / PCB_SIZE * PCB_SIZE
        }
    };
}



#[repr(u8)]
#[derive(PartialEq)]
pub enum Status {
    Ready,
    Running,
}


// ready -> running
#[repr(C)]
pub struct PCB {
    pointers: [usize; 4],
    reg_ctx: [u32; REG_CTX_LEN],
    rt: Routine,
    entry: usize,
    status: Status,
    priority: u8,
    ticks: u8,
    elapsed_ticks: u32,
    name_len: u8,
    name_buf: [u8; 16],
    magic: u32,
}

impl Node for PCB {
    fn pointers_mut(&mut self) -> &mut [usize] {
        &mut self.pointers
    }

    fn pointers(&self) -> &[usize] {
        &self.pointers
    }
}

impl PCB {
    pub fn new(rt: Routine, name: &str, priority: u8, off: usize) -> &'static mut Self {
        let p: &'static mut PCB = unsafe { core::mem::transmute(off) };
        let len = p.name_buf.len().min(name.as_bytes().len());
        p.rt = rt;
        p.name_len = len as u8;
        p.name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        p.ticks = priority;
        p.priority = priority;
        p.status = Ready;
        p.magic = STACK_MAGIC;

        println!("new eip = 0x{:08X}", p.rt as usize);
        p
    }

    pub fn status_mut(&mut self) -> &mut Status {
        &mut self.status
    }

    fn overflow(&self) -> bool {
        self.magic != STACK_MAGIC
    }

    pub fn name(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name_buf[..self.name_len as usize]) }
    }

    #[inline]
    pub fn off(&self) -> usize {
        self as *const Self as usize
    }

    pub fn stack(&self) -> usize {
        self.off() + PCB_SIZE
    }
}

// get current process control block
pub fn current_pcb() -> &'static mut PCB {
    let p = cur_pcb!();
    unsafe { &mut *(p as usize as *mut _) }
}

pub fn new_thread(rt: Routine, name: &str, priority: u8) {
    let pcb_off = pg_alloc(Pool::KERNEL, 1).unwrap();
    let pcb = PCB::new(rt, name, priority, pcb_off);
    all().append(pcb);
}

pub fn init() {
    data::init();

    // register handler
    crate::idt::register(0x20, schedule);
}


// process scheduler
pub fn schedule() {
    // get current pcb
    let cur = current_pcb();

    // check if overflow
    assert!(!cur.overflow(), "stack of thread {} overflow!", cur.name());

    let t = ticks();
    unsafe {
        *t = t.unchecked_add(1);
        cur.elapsed_ticks = cur.elapsed_ticks.unchecked_add(1)
    }

    if cur.ticks != 0 {
        cur.ticks -= 1;
        return;
    }


    cur.ticks = cur.priority;
    // switch to another thread
    let l = all();
    let h = l.pop_head();

    if h.is_none() {
        return;
    }


    let p = h.unwrap();

    // save ctx
    let ctx = reg_ctx();
    cur.reg_ctx.copy_from_slice(ctx);

    if p.status == Ready {
        p.reg_ctx.copy_from_slice(ctx);

        p.reg_ctx.reset_general();
        *p.reg_ctx.eip() = p.rt as usize as u32;
        *p.reg_ctx.esp() = p.stack() as u32;
        *p.reg_ctx.ebp() = p.stack() as u32;
        p.status = Running;
    }

    // restore context
    ctx.copy_from_slice(&p.reg_ctx);
}
