use rlib::alloc_static;
use rlib::link::{LinkedList, Node};

use crate::{Pool, print, println};
use crate::asm::{IntCtx, reg_ctx, REG_CTX_LEN, switch};
use crate::err::SE;
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc};
use crate::thread::data::all;
use crate::thread::Status::{Ready, Running};

mod data;
mod reg;

pub type Routine = extern "C" fn(args: usize);

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

pub extern "C" fn entry() {
    crate::asm::sti();
    let cur = current_pcb();
    let fun = cur.rt;
    fun(cur.args);
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
    stack: usize,
    pointers: [usize; 4],
    reg_ctx: [u32; REG_CTX_LEN],
    rt: Routine,
    args: usize,
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
    pub fn new(rt: Routine, args: usize, name: &str, priority: u8, off: usize) -> &'static mut Self {
        let p: &'static mut PCB = cst!(off);
        let len = p.name_buf.len().min(name.as_bytes().len());

        p.stack = off + PCB_SIZE;
        p.rt = rt;
        p.args = args;
        p.name_len = len as u8;
        p.name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        p.ticks = priority;
        p.priority = priority;
        p.status = Ready;
        p.magic = STACK_MAGIC;
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
    cst!(p)
}

pub fn new_thread(rt: Routine, args: usize, name: &str, priority: u8) {
    let pcb_off = pg_alloc(Pool::KERNEL, 1).unwrap();
    let pcb = PCB::new(rt, args, name, priority, pcb_off);
    all().append(pcb);
}

pub fn init() {
    data::init();

    // register handler
    crate::idt::register(0x20, schedule);
}

static mut SWITCH_CNT: u64 = 0;

fn switch_cnt() -> &'static mut u64 {
    unsafe { &mut SWITCH_CNT }
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
    let l = all();
    l.append(cur);

    let n = l.pop_head().unwrap();

    unsafe {
        *switch_cnt() += 1;
    }
    println!("switch = {}", unsafe { SWITCH_CNT });
    switch(cur.off(), n.off());
}
