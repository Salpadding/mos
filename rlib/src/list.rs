use core::marker::PhantomData;

///  node of bi direction linked list
pub struct Node {
    prev: usize,
    next: usize,
}

impl Node {
    fn null() -> Self {
        Self {
            prev: 0,
            next: 0,
        }
    }

    fn prepend(&mut self, n: &Node) {
        let p = unsafe {
            n as *const _ as usize
        };
        self.prev = p;
    }

    fn append(&mut self, n: &Node) {
        let p = unsafe {
            n as *const _ as usize
        };
        self.next = p;
    }
}

pub struct List {
    head: Node,
    tail: Node,
}

impl List {
   fn new() -> Self {
       Self {
           head: Node::null(),
           tail: Node::null(),
       }
   }

    fn init(&mut self) {
        self.head.append(&self.tail);
        self.tail.prepend(&self.head);
    }
}

macro_rules! di {
    ($f: ident) => {
        pub fn $f(&self) -> Option<&'static mut Node> {
            if self.$f == 0 {
                None
            } else {
                Some(
                    unsafe { &mut *(self.$f as *mut _) }
                )
            }
        }
    };
}

impl Node {
    di!(next);
    di!(prev);
}
