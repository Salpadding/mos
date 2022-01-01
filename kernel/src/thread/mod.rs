use rlib::alloc_static;
use rlib::link::{LinkedList, Node};

use crate::asm::{switch, REG_CTX_LEN};
use crate::err::SE;
use crate::mem::{fill_zero, pg_alloc, PAGE_SIZE};
use crate::thread::data::{all, ready};
use crate::thread::Status::{Ready, Running};
use crate::{print, println, Pool};
use crate::thread::reg::IntCtx;

use self::reg::KernelCtx;

mod data;
pub mod reg;
pub mod sync;

pub type Routine = extern "C" fn(args: usize);
pub const MAIN_PRIORITY: u8 = 31;

pub const PCB_PAGES: usize = 1;
const STACK_MAGIC: u32 = 0x238745ea;
pub const PCB_SIZE: usize = PCB_PAGES * PAGE_SIZE;

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

pub extern "C" fn entry(fun: Routine, args: usize) {
    crate::asm::sti();
    fun(args);
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum Status {
    Ready,
    Running,
    Blocked,
    Waiting,
    Hanging,
    Died
}

// ready -> running
#[repr(C)]
pub struct PCB {
    stack: usize,
    pointers: [usize; 4],
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
    pub fn new(
        name: &str,
        priority: u8,
        off: usize,
    ) -> &'static mut Self {
        let p: &'static mut PCB = cst!(off);
        let len = p.name_buf.len().min(name.as_bytes().len());

        p.stack = off + PCB_SIZE;
        p.name_len = len as u8;
        p.name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        p.ticks = priority;
        p.priority = priority;
        p.status = Ready;
        p.magic = STACK_MAGIC;
        p
    }

    pub fn init(&mut self, rt: Routine, arg: usize) {
        self.stack -= core::mem::size_of::<crate::thread::reg::IntCtx>();
        self.stack -= core::mem::size_of::<crate::thread::reg::KernelCtx>();
        let k_ctx = self.kernel_ctx();
        k_ctx.func = rt as usize as u32;
        k_ctx.eip = entry as usize as u32;
        k_ctx.arg = arg as u32;
    }

    fn kernel_ctx(&self) -> &'static mut KernelCtx {
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
}

// get current process control block
pub fn current_pcb() -> &'static mut PCB {
    let p = cur_pcb!();
    cst!(p)
}

pub fn new_thread(rt: Routine, args: usize, name: &str, priority: u8) {
    let pcb_off = pg_alloc(Pool::KERNEL, 1).unwrap();
    let pcb = PCB::new(name, priority, pcb_off);
    pcb.init(rt, args);
    ready().append(pcb);
    all().append(pcb);
}

pub fn init() {
    data::init();

    // add main thread to all list
    let main = current_pcb();
    all().append(main);

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
    schedule();
}

// process scheduler
pub fn schedule() {
    let cur = current_pcb();
    let off = cur.off();
    let rd = ready();

    if cur.status == Status::Running {
        assert!(!rd.raw_iter().any(|x| x == off), "thread in ready");
        cur.status = Status::Ready;
        cur.ticks = cur.priority;
        rd.append(cur);
    }

    assert!(!rd.is_empty(), "ready is empty");
    let n = rd.pop_head().unwrap();
    n.status = Status::Running;

    switch(cur.off(), n.off());
}
