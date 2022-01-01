use crate::int::{disable_int, set_int};
use crate::thread::{current_pcb, PCB, schedule, Status};
use crate::thread::data::{all, ready};
use rlib::link::LinkedList;

pub struct Semaphore<'a> {
    value: u32,
    waiters: &'a mut LinkedList<PCB>,
}

impl Semaphore<'_> {
    // p operation
    pub fn p(&mut self) {
        let old = disable_int();
        let cur = current_pcb();
        let off = cur.off();
        while self.value == 0 {
            assert!(!self.waiters.raw_iter().any(|x| x == off), "duplicate p op");
            self.waiters.append(cur);
            block(Status::Blocked);
        }
        self.value -= 1;
        set_int(old);
    }

    pub fn v(&mut self) {
        let old = disable_int();
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
        return
    }

    let rd = ready();
    assert!(!rd.raw_iter().any(|x| x == off), "target thread not blocked");
    pcb.status = Status::Ready;
    rd.push_head(pcb);
    set_int(old);
}