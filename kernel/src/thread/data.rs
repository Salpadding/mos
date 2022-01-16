use rlib::alloc_static;
use rlib::link::LinkedList;

use crate::thread::PCB;
use crate::thread::PCB_PADDING;

alloc_static!(ALL, all, LinkedList<PCB, PCB_PADDING>);
alloc_static!(READY, ready, LinkedList<PCB, PCB_PADDING>);

pub fn init() {
    // general tag
    let all = all();
    all.init(0, 1);

    let r = ready();
    r.init(2, 3);
}