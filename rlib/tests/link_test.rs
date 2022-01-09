use std::fmt::{Debug, Display, Formatter};

use rlib::alloc_static;
use rlib::link::{LinkedList, Node};

#[derive(Default)]
pub struct LNode {
    pts: [usize; 8],
    id: usize,
}

impl Debug for LNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(node, id = {})", self.id)
    }
}

impl Node for LNode {
    fn pointers_mut(&mut self) -> &mut [usize] {
        &mut self.pts
    }

    fn pointers(&self) -> &[usize] {
        &self.pts
    }
}

alloc_static!(HD, hd, LNode);
alloc_static!(TL, tl, LNode);
alloc_static!(N0, n0, LNode);
alloc_static!(N1, n1, LNode);
alloc_static!(N2, n2, LNode);
alloc_static!(N3, n3, LNode);


#[repr(C)]
struct O {
    d: u8,
    e: u8,
    f: u8,
}

#[test]
fn test() {
    println!("align = {}", core::mem::align_of::<u16>());
    println!("align = {}", core::mem::align_of::<O>());
    println!("size = {}", core::mem::size_of::<O>());
    let mut li: LinkedList<LNode, 256> = LinkedList::default();
    li.init(0, 1);

    for x in [n0(), n1(), n2(), n3()].into_iter().enumerate() {
        x.1.id = x.0
    }

    // append n0, n1
    li.append(n0());
    li.append(n1());

    let v: Vec<_> = li.iter().collect();
    println!("{:?}", v);
    println!("{}", li.len());


    // prepend n2
    li.push_head(n2());
    let v: Vec<_> = li.iter().collect();
    println!("{:?}", v);

    // pop head
    let pop = li.pop_head();
    println!("{:?}", pop);
    let v: Vec<_> = li.iter().collect();
    println!("{:?}", v);


    li.append(pop.unwrap());
    let v: Vec<_> = li.iter().collect();
    println!("{:?}", v);
}

const LEN: usize = 32;
static mut X: [u32; LEN] = [0; LEN];

#[test]
fn test1() {
    let x = unsafe {
        core::slice::from_raw_parts_mut(
            X.as_mut_ptr().add(LEN / 4),
            LEN / 4,
        )
    };

    x.fill(1);
    let mut y: [u32; LEN / 4] = [0; LEN / 4];


    unsafe {
        y.copy_from_slice(x);
    }

    println!("{:?}", y);
}