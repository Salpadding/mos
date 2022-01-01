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
        r.sem.waiters = waiters;
        r.sem.value = 1;
        r
    }

    pub fn lock(&mut self) {
        let cur = current_pcb();

        println!("cur name = {}", cur.name());
        println!("holder is some = {}", self.holder.is_some());
        if self.holder.is_some() && self.holder.as_ref().unwrap().off() == cur.off() {
            self.repeats += 1;
            return;
        }
        println!("try to p");
        self.sem.p();
        println!("p success");
        self.holder = Some(cur);
        self.repeats += 1;
    }

    pub fn unlock(&mut self) {
        let cur = current_pcb();
        println!("unlock holder is some = {}", self.holder.is_some());
        assert_eq!(self.holder.as_ref().unwrap().off(), cur.off(), "unlock without lock");

        if self.repeats > 1 {
            self.repeats -= 1;
            return;
        }

        assert_eq!(self.repeats, 1, "repeats != 0");
        self.holder = None;
        self.repeats = 0;
        println!("try sem.v()");
        self.sem.v();
    }
}

impl Semaphore {
    // p operation
    pub fn p(&mut self) {
        let old = disable_int();
        let cur = current_pcb();
        let off = cur.off();
        while self.value == 0 {
            assert!(!self.waiters.raw_iter().any(|x| x == off), "duplicate p op");
            self.waiters.append(cur);
            println!("waiters length = {}", self.waiters.len());
            block(Status::Blocked);
        }
        self.value -= 1;
        set_int(old);
    }

    pub fn v(&mut self) {
        println!("v()");
        let old = disable_int();
        println!("int disabled");
        println!("waiters length = {}", self.waiters.len());
        if !self.waiters.is_empty() {
            let blocked = self.waiters.pop_head().unwrap();
            unblock(blocked);
        }

        self.value += 1;
        set_int(old);
    }
}


pub fn block(status: Status) {
    assert!(status == Status::Blocked || status == Status::Waiting || status == Status::Hanging);
    let old = disable_int();
    let cur = current_pcb();
    cur.status = status;
    schedule();
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