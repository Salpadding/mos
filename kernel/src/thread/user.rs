use crate::mem::pg_alloc;
use crate::Pool;
use crate::thread::{PCB, Routine};

fn create_pd() {
    let pg = pg_alloc(Pool::KERNEL, 1).unwrap();

}

pub fn create(rt: Routine, args: usize, name: &str, priority: u8) {
    let pcb_off = pg_alloc(Pool::KERNEL, 1).unwrap();
    let pcb = PCB::new(name, priority, pcb_off);
    pcb.init(rt, args);
}