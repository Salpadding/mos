use crate::err::SE;
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc};
use crate::Pool;
use crate::thread::Status::Running;

const PCB_PAGES: usize = 1;

#[repr(u8)]
pub enum Status {
    Running,
    Ready,
    Blocked,
    Waiting,
    Hanging,
    Died,
}

/// interrupt stack
struct IntStack {
    // eax, ebx, ecx, edx, esp, ebp, esi, edi,
    g_regs: [u32; 8],
    // es, cs, ss, ds, fs, gs
    s_regs: [u16; 6],

    e_code: u32,
    caller: usize,
    cs: u16,
    e_flags: u32,
    esp: usize,
    ss: usize,
}

/// thread stack
struct ThreadStack {
    // ebp, ebx, edi, esi
    regs: [u32; 4],

    caller: usize,
}

/// process control block, 4kb
struct PCB {
    off: usize,
    status: Status,
    priority: u8,
    name: [u8; 8],
    magic: u32,
}

impl PCB {
    fn new(name: &str, priority: u8, off: usize) -> &'static mut PCB {
        fill_zero(off, PCB_PAGES * PAGE_SIZE);
        let p: &'static mut PCB = unsafe {
            core::mem::transmute(off)
        };

        let len = p.name.len().min(name.len());
        p.name[..len].copy_from_slice(&name.as_bytes()[..len]);
        p.priority = priority;
        p.status = Running;
        p.off = off;
        p
    }

    fn int_stack(&self) -> &'static mut IntStack {
        let p = self.off + PCB_PAGES * PAGE_SIZE - core::mem::size_of::<IntStack>();
        unsafe { &mut *(p as *mut _) }
    }

    fn th_stack(&self) -> &'static mut ThreadStack {
        let p = self.off + PCB_PAGES * PAGE_SIZE - core::mem::size_of::<IntStack>()
            - core::mem::size_of::<ThreadStack>();
        unsafe { &mut *(p as *mut _) }
    }

    fn init(&mut self, rt: Routine, ctx: *mut u8) {}
}

pub type Routine = extern "C" fn(ctx: *mut u8);

pub fn start(r: Routine) -> Result<(), SE> {
    // allocate a page for process control block
    let pcb = pg_alloc(Pool::KERNEL, PCB_PAGES)?;

    Ok(())
}