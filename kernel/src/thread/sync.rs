use rlib::link::LinkedList;

use crate::int::{disable_int, set_int};
use crate::thread::data::{all, ready};
use crate::thread::{current_pcb, schedule, Status, PCB};
use crate::{c_println, print, println };

#[repr(C)]
pub struct Semaphore {
    value: u32,
    waiters: &'static mut LinkedList<PCB>,
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

impl Lock {
    pub fn new(off: usize, len: usize) -> &'static mut Self {
        let lock_len = (core::mem::size_of::<Lock>() + 7) / 8 * 8;
        assert!(len >= lock_len + LinkedList::<PCB>::alloc_size());
        let waiters_off = off + lock_len;
        let waiters = LinkedList::new(waiters_off, 2, 3);
        let r: &'static mut Self = cst!(off);
        r.holder = None;
        r.sem = Semaphore { value: 1, waiters };
        r.repeats = 0;
        r
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
