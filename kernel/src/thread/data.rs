use rlib::alloc_static;
use rlib::link::LinkedList;

use crate::thread::PCB;

alloc_static!(HD0, hd0, PCB);
alloc_static!(TL0, tl0, PCB);
alloc_static!(HD1, hd1, PCB);
alloc_static!(TL1, tl1, PCB);

alloc_static!(ALL, all, LinkedList<PCB>);
alloc_static!(READY, ready, LinkedList<PCB>);

pub fn init() {
    let all = all();
    all.init(0, 1, hd0(), tl0());

    let r = ready();
    r.init(2, 3, hd1(), tl1());
}