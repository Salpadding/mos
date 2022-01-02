use rlib::link::LinkedList;

use crate::int::{disable_int, set_int};
use crate::println;
use crate::thread::{current_pcb, PCB, schedule, Status};
use crate::thread::data::{all, ready};

pub struct Semaphore {
    value: u32,
    waiters: &'static mut LinkedList<PCB>,
}

pub struct Lock {
    holder: Option<&'static PCB>,
    sem: Semaphore,
    repeats: u32,
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

    pub fn lock(&mut self) {
        let cur = current_pcb();

        if self.holder.is_some() && self.holder.as_ref().unwrap().off() == cur.off() {
            self.repeats += 1;
            return;
        }
        self.sem.p();
        self.holder = Some(cur);
        self.repeats += 1;
    }

    pub fn unlock(&mut self) {
        let cur = current_pcb();
        assert_eq!(self.holder.as_ref().unwrap().off(), cur.off(), "unlock without lock");

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
    pub fn p(&mut self) {
        let old = disable_int();
        let cur = current_pcb();
        println!("{}: p()", cur.name());
        while self.value == 0 {
            assert!(!self.waiters.raw_iter().any(|x| x == cur.off()), "duplicate p op");
            self.waiters.append(cur);
            println!("{}: block", cur.name());
            block(Status::Blocked);
            println!("{}: ret from block value = {}", cur.name(), self.value);
        }
        self.value -= 1;
        println!("{}: p() success", cur.name());
        set_int(old);
    }

    pub fn v(&mut self) {
        let old = disable_int();

        let cur = current_pcb();
        println!("{}: v()", cur.name());
        if !self.waiters.is_empty() {
            let blocked = self.waiters.pop_head().unwrap();
            println!("{}: unblock {}", cur.name(), blocked.name());
            unblock(blocked);
        }

        self.value += 1;
        println!("{}: v() success", cur.name());
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
    assert!(pcb.status == Status::Blocked || pcb.status == Status::Waiting || pcb.status == Status::Hanging);

    if pcb.status == Status::Ready {
        set_int(old);
        return;
    }

    let rd = ready();
    assert!(!rd.raw_iter().any(|x| x == off), "target thread not blocked");
    pcb.status = Status::Ready;
    rd.push_head(pcb);
    set_int(old);
}