use core::marker::PhantomData;

///  node of bi direction linked list
pub struct Node {
    prev: usize,
    next: usize,
}

impl Node {
    pub fn new() -> Self {
        Self {
            prev: 0,
            next: 0,
        }
    }

    fn set_prev(&mut self, n: &Node) {
        let p = unsafe {
            n as *const _ as usize
        };
        self.prev = p;
    }

    fn set_next(&mut self, n: &Node) {
        let p = unsafe {
            n as *const _ as usize
        };
        self.next = p;
    }

    pub fn eq(&self, o: &Self) -> bool {
        self as *const _ as usize == o as *const _ as usize
    }

    pub fn detach(&mut self) {
        let p = self.prev().unwrap();
        let n = self.next().unwrap();

        p.set_next(n);
        n.set_prev(p);
    }

    fn prepend(&mut self, n: &mut Node) {
        let u = self.prev().unwrap();
        u.set_next(n);
        n.set_prev(u);
        n.set_next(self);
        self.set_prev(n);
    }
}

pub struct List {
    head: Node,
    tail: Node,
}

impl List {
    pub fn new() -> Self {
        Self {
            head: Node::new(),
            tail: Node::new(),
        }
    }

    pub fn init(&mut self) {
        self.head.set_next(&self.tail);
        self.tail.set_prev(&self.head);
    }

    fn is_empty(&self) -> bool {
        self.head.next == &self.tail as *const _ as usize
    }

    fn len(&self) -> usize {
        let mut l = 0usize;

        self.traverse(|x| -> bool {
            l += 1;
            true
        });

        l
    }

    pub fn prepend(&mut self, n: &mut Node) {
        let m = self.head.next().unwrap();
        m.prepend(n);
    }

    pub fn append(&mut self, n: &mut Node) {
        self.tail.prepend(n);
    }

    pub fn pop_head(&mut self) -> Option<&'static mut Node> {
        let n = self.head.next();

        if n.is_none() { None } else {
            let m = n.unwrap();
            m.detach();
            Some(m)
        }
    }

    pub fn contains(&self, n: &Node) -> bool {
        let mut ret = false;
        self.traverse(|m| {
            ret = m.eq(n);
            !m.eq(n)
        });
        ret
    }

    pub fn traverse<F: FnMut(&Node) -> bool>(&self, mut f: F) {
        let mut cur = self.head.next();
        while cur.is_some() {
            let u = cur.unwrap();
            if u.eq(&self.tail) { break; }
            if !f(u) { break; }
            cur = u.next();
        }
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

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let mut li = super::List::new();
        li.init();

        let mut n = super::Node::new();

        li.prepend(&mut n);
        println!("li.len() = {}", li.len());

        li.pop_head();
        println!("li.len() = {}", li.len());
    }
}