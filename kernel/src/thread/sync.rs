use rlib::link::LinkedList;

use crate::{c_println, print, println};
use crate::int::{disable_int, set_int};
use crate::thread::{current_pcb, PCB, schedule, Status, ticks};
use crate::thread::data::{all, ready};
use crate::thread::PCB_PADDING;
use crate::timer::MIL_SECONDS_PER_INT;

#[repr(C)]
pub struct SpinLock {
    data: u32
}

impl SpinLock {
    pub fn lock(&self) {
        let p = self as *const _ as usize;
        unsafe {
            asm!("2:", "xchg eax, [{0}]", "test eax, eax", "jnz 2b", in(reg) p, in("eax") 1)
        }
    }

    pub fn unlock(&self) {
        let p = self as *const _ as usize;
        unsafe {
            asm!("xchg eax, [{0}]", in(reg) p, in("eax") 0);
        }
    }
}

pub fn sleep_ticks(t: usize) {
    let start = *ticks();
    while *ticks() - start < t as u32 {
        th_yield();
    }
}

pub fn sleep_mils(t: u32) {
    let ticks = t / MIL_SECONDS_PER_INT;
    sleep_ticks(ticks as usize);
}

#[repr(C)]
pub struct Semaphore {
    pub value: u32,
    pub waiters: LinkedList<PCB, PCB_PADDING>,
}

#[repr(C)]
pub struct Lock {
    holder: Option<&'static PCB>,
    sem: Semaphore,
    repeats: u32,
}

pub struct Guard {
    lock: usize,
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.unlock();
    }
}

impl Guard {
    pub fn unlock(&mut self) {
        let p = self.lock;
        self.lock = 0;

        if p == 0 {
            return;
        }

        let l: &'static mut Lock = cst!(p);
        l.unlock();
    }
}

pub fn th_yield() {
   let cur = current_pcb();
    let old = disable_int();
    assert!(!ready().raw_iter().any(|x| x == cur.off()), "cur shouldn't in ready");
    ready().append(cur);
    cur.status = Status::Ready;
    schedule("yield");
    set_int(old);
}

impl Lock {
    pub fn new(off: usize, len: usize) -> &'static mut Self {
        let lock_len = core::mem::size_of::<Lock>();
        assert!(len >= lock_len);
        let r: &'static mut Self = cst!(off);
        r.init();
        r
    }

    pub fn init(&mut self) {
        self.holder = None;
        self.sem.value = 1;
        self.sem.waiters.init(2, 3);
        self.repeats = 0;
    }

    #[no_mangle]
    pub fn lock(&mut self) -> Guard {
        let cur = current_pcb();

        if self.holder.is_some() && self.holder.as_ref().unwrap().off() == cur.off() {
            self.repeats += 1;
            return Guard { lock: self as *const _ as usize };
        }
        self.sem.p();
        self.holder = Some(cur);
        self.repeats += 1;
        return Guard { lock: self as *const _ as usize };
    }

    #[no_mangle]
    pub fn unlock(&mut self) {
        let cur = current_pcb();
        assert_eq!(
            self.holder.as_ref().unwrap().off(),
            cur.off(),
            "unlock without lock"
        );

        if self.repeats > 1 {
            self.repeats -= 1;
            return;
        }

        assert_eq!(self.repeats, 1, "repeats != 0");
        self.holder = None;
        self.repeats = 0;
        self.sem.v();
    }
}

impl Semaphore {
    // p operation
    #[inline(never)]
    #[no_mangle]
    pub fn p(&mut self) {
        let old = disable_int();
        let cur = current_pcb();
        debug!("{}: p()", cur.name());
        while self.value == 0 {
            assert!(
                !self.waiters.raw_iter().any(|x| x == cur.off()),
                "duplicate p op"
            );
            self.waiters.append(cur);
            debug!("{}: block", cur.name());
            block(Status::Blocked);
            debug!("{}: ret from block value = {}", cur.name(), self.value);
        }
        self.value -= 1;
        debug!("{}: p() success", cur.name());
        set_int(old);
    }

    #[inline(never)]
    #[no_mangle]
    pub fn v(&mut self) {
        let old = disable_int();

        let cur = current_pcb();
        debug!("{}: v()", cur.name());
        if !self.waiters.is_empty() {
            let blocked = self.waiters.pop_head().unwrap();
            debug!("{}: unblock {}", cur.name(), blocked.name());
            unblock(blocked);
        }

        self.value += 1;
        debug!("{}: v() success", cur.name());
        set_int(old);
    }
}

pub fn block(status: Status) {
    assert!(status == Status::Blocked || status == Status::Waiting || status == Status::Hanging);
    let old = disable_int();
    let cur = current_pcb();
    cur.status = status;
    schedule("block");
    set_int(old);
}

pub fn unblock(pcb: &'static mut PCB) {
    let old = disable_int();
    let off = { pcb.off() };
    assert!(
        pcb.status == Status::Blocked
            || pcb.status == Status::Waiting
            || pcb.status == Status::Hanging
    );

    if pcb.status == Status::Ready {
        set_int(old);
        return;
    }

    let rd = ready();
    assert!(
        !rd.raw_iter().any(|x| x == off),
        "target thread not blocked"
    );
    pcb.status = Status::Ready;
    rd.push_head(pcb);
    set_int(old);
}

