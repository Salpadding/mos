use std::fmt::{Debug, Display, Formatter};
use rlib::link::{LinkedList, Node};
use rlib::{alloc_static, alloc_statics};

#[derive(Default)]
pub struct LNode {
    pts: [usize; 8],
    id: usize,
}

impl Debug for LNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f, "[node, id = {}]", self.id)
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

#[test]
fn test() {
    let mut li: LinkedList<LNode> = LinkedList::default();
    li.init(0, 1, hd(), tl());

    li.append(n0());

    let v: Vec<_> = li.iter().collect();

    println!("{:?}", v);
}