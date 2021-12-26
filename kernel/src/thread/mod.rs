use crate::err::SE;
use crate::mem::{fill_zero, PAGE_SIZE, pg_alloc};
use crate::Pool;
use crate::thread::Status::Running;

const PCB_PAGES: usize = 1;
const STACK_MAGIC: u32 = 0x55aa55aa;

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
struct IntBlock {
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
#[repr(packed)]
struct ThreadBlock {
    // ebp, ebx, edi, esi
    regs: [u32; 4],
    eip: usize,
    ret: Routine,
    rt: Routine,
    args: usize,
}

/// process control block, 4kb
pub struct PCB {
    status: Status,
    priority: u8,
    name: [u8; 8],
    magic: u32,
}

impl PCB {
    fn new(name: &str, priority: u8, off: usize) -> &'static mut Self {
        fill_zero(off, PCB_PAGES * PAGE_SIZE);
        let p: &'static mut PCB = unsafe {
            core::mem::transmute(off)
        };

        let len = p.name.len().min(name.len());
        p.name[..len].copy_from_slice(&name.as_bytes()[..len]);
        p.priority = priority;
        p.status = Running;
        p.magic = STACK_MAGIC;
        p
    }

    fn overflow(&self) -> bool {
        self.magic == STACK_MAGIC
    }
    #[inline]
    fn off(&self) -> usize {
        self as *const Self as usize
    }

    fn int_block(&self) -> &'static mut IntBlock {
        let p = self.off() + PCB_PAGES * PAGE_SIZE - core::mem::size_of::<IntBlock>();
        unsafe { &mut *(p as *mut _) }
    }

    fn th_block(&self) -> &'static mut ThreadBlock {
        let p = self.off() + PCB_PAGES * PAGE_SIZE - core::mem::size_of::<IntBlock>()
            - core::mem::size_of::<ThreadBlock>();
        unsafe { &mut *(p as *mut _) }
    }

    fn stack(&self) -> usize {
        self.off() + PCB_PAGES * PAGE_SIZE - core::mem::size_of::<IntBlock>()
            - core::mem::size_of::<ThreadBlock>()
    }

    fn init(&mut self, rt: Routine, args: usize) {
        let th_s = self.th_block();
        th_s.eip = unsafe { kernel_thread as usize };
        th_s.rt = rt;
        th_s.args = args;

    }
}

pub type Routine = extern "C" fn(arg: usize);

pub extern "C" fn kernel_thread(f: Routine, arg: usize) {
    f(arg);
}

pub fn init() {

}

pub fn start(name: &str, priority: u8, r: Routine, args: usize) -> Result<&'static mut PCB, SE> {
    // allocate a page for process control block
    let pcb = pg_alloc(Pool::KERNEL, PCB_PAGES)?;
    let pcb = PCB::new(name, priority, pcb);
    pcb.init(r, args);

    unsafe {
        asm!(
        // reset esp
        "mov esp, {}",
        in(reg) pcb.stack()
        );
    }
    Ok(pcb)
}