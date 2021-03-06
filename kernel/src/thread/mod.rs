use rlib::alloc_static;
use rlib::link::{LinkedList, Node};

use crate::asm::{switch, REG_CTX_LEN, SELECTOR_K_DATA};
use crate::err::SE;
use crate::mem::arena::{BlkDesc, DESC_CNT};
use crate::mem::page::PDE_START;
use crate::mem::PagePool;
use crate::mem::PageTable;
use crate::mem::{fill_zero, pg_alloc, VPool, PAGE_SIZE, PT_LEN};
use crate::thread::data::{all, ready};
use crate::thread::reg::IntCtx;
use crate::thread::sync::{block, unblock};
use crate::thread::tss::esp0;
use crate::thread::Status::{Ready, Running};
use crate::{c_println, print, println, Pool};

use self::reg::KernelCtx;

pub const DEBUG: bool = false;

macro_rules! debug {
    ($($arg:tt)*) => {
        if !$crate::int::int_enabled() && unsafe { $crate::thread::DEBUG } {
            assert!(!$crate::int::int_enabled(), "int enabled");
            c_println!($($arg)*);
        }
    };
}

pub mod data;
pub mod reg;
pub mod sync;
pub mod tss;
pub mod user;

pub type Routine = extern "C" fn(args: usize);
pub type Entry = extern "C" fn(rt: Routine, args: usize);

pub const DEFAULT_PRIORITY: u8 = 32;
pub const MAIN_PRIORITY: u8 = DEFAULT_PRIORITY;

pub const PCB_PAGES: usize = 1;
const STACK_MAGIC: u32 = 0x238745ea;
pub const PCB_SIZE: usize = PCB_PAGES * PAGE_SIZE;
pub const PCB_PADDING: usize = 128;

static mut IDLE: usize = 0;

pub fn idle_thread() -> &'static mut PCB {
    cst!(IDLE)
}

static mut TICKS: u32 = 0;

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

/// idle thread
pub extern "C" fn idle(args: usize) {
    loop {
        block(Status::Blocked);
        //c_println!("hlt0");
        unsafe {
            asm!("sti", "hlt");
        }
        //c_println!("hlt1");
    }
}

/// entry of kernel thread
pub extern "C" fn entry(fun: Routine, args: usize) {
    crate::asm::sti();
    fun(args);
}

#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum Status {
    Ready,
    Running,
    Blocked,
    Waiting,
    Hanging,
    Died,
}

// ready -> running
#[repr(C)]
pub struct PCB {
    stack: usize,
    pointers: [usize; 4],
    pub status: Status,
    priority: u8,
    pub ticks: u8,
    elapsed_ticks: u32,
    name_len: u8,
    name_buf: [u8; 16],

    // page directory, 0 for kernel thread
    pub pd: usize,

    // virtual memory pool, for user process
    v_pool: VPool,
    pub desc: [BlkDesc; DESC_CNT],
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
    pub fn new(name: &str, priority: u8, off: usize) -> &'static mut Self {
        let p: &'static mut PCB = cst!(off);
        let len = p.name_buf.len().min(name.as_bytes().len());

        p.stack = off + PCB_SIZE;
        p.name_len = len as u8;
        p.name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        p.pd = 0;
        p.ticks = priority;
        p.priority = priority;
        p.status = Ready;
        p.magic = STACK_MAGIC;
        p
    }

    pub fn init(&mut self, entry: Entry, rt: Routine, arg: usize) {
        self.stack -= core::mem::size_of::<crate::thread::reg::IntCtx>();
        self.stack -= core::mem::size_of::<crate::thread::reg::KernelCtx>();
        let k_ctx = self.kernel_ctx();
        k_ctx.func = rt as usize as u32;
        k_ctx.eip = entry as usize as u32;
        k_ctx.ds = SELECTOR_K_DATA as u32;
        k_ctx.es = SELECTOR_K_DATA as u32;
        k_ctx.arg = arg as u32;
    }

    #[inline]
    pub fn user(&self) -> bool {
        self.pd != 0
    }

    #[inline]
    fn kernel_ctx(&self) -> &'static mut KernelCtx {
        cst!(self.stack)
    }

    #[inline]
    fn int_ctx(&self) -> &'static mut IntCtx {
        cst!(self.stack)
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

    pub fn stack_off(&self) -> usize {
        self.off() + PCB_SIZE
    }

    pub fn v_pool(&mut self) -> &mut VPool {
        &mut self.v_pool
    }

    pub fn page_dir(&self) -> Option<PageTable> {
        if self.pd == 0 {
            None
        } else {
            Some(unsafe { core::slice::from_raw_parts_mut(self.pd as *mut _, PT_LEN) })
        }
    }
}

// get current process control block
pub fn current_pcb() -> &'static mut PCB {
    let p = cur_pcb!();
    cst!(p)
}

pub fn new_thread(rt: Routine, args: usize, name: &str, priority: u8) -> &'static mut PCB {
    let pcb_off = pg_alloc(Pool::KERNEL, 1, true).unwrap();
    let pcb = PCB::new(name, priority, pcb_off);
    pcb.init(entry, rt, args);
    ready().append(pcb);
    all().append(pcb);
    pcb
}

pub fn init() {
    data::init();

    // add main thread to all list
    let main = current_pcb();
    all().append(main);

    // create idle thread
    unsafe { IDLE = new_thread(idle, 0, "idle", DEFAULT_PRIORITY / 2) as *const _ as usize };

    // register handler
    crate::int::register(0x20, handle_int);
}

fn handle_int(ctx: &'static mut IntCtx) {
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
    schedule("int");
}

// process scheduler
pub fn schedule(reason: &str) {
    assert!(!crate::int::int_enabled(), "int enabled");
    let cur = current_pcb();
    let off = cur.off();
    let rd = ready();

    //c_println!("rd.is_empty() = {:?}", rd.is_empty());
    if cur.status == Status::Running {
        assert!(!rd.raw_iter().any(|x| x == off), "thread in ready");
        cur.status = Status::Ready;
        cur.ticks = cur.priority;
        rd.append(cur);
    } 

    if rd.is_empty() {
        // wake idle thread if ready is empty
        unblock(idle_thread());
    }

    let n = rd.pop_head().unwrap();
    //c_println!("next = {}", n.name());
    n.status = Status::Running;

    debug!(
        "switch from {} to {} reason = {}",
        cur.name(),
        n.name(),
        reason
    );

    let pd: usize = if n.user() { n.pd } else { PDE_START };

    unsafe {
        asm!("mov cr3, {}", in(reg) pd);
    }

    if n.user() {
        *esp0() = (n.off() + PCB_SIZE) as u32;
    }

    switch(cur.off(), n.off());
}
