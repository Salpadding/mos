use core::marker::PhantomData;
use core::ops::Deref;

pub trait Node: Sized {
    fn pointers_mut(&mut self) -> &mut [usize];
    fn pointers(&self) -> &[usize];

    fn ref_at(&self, i: usize) -> Option<&'static mut Self> {
        let p = self.pointers()[i];

        if p == 0 {
            None
        } else {
            Some(unsafe { &mut *(self.pointers()[i] as *mut Self) })
        }
    }
}

pub struct LinkedList<T: Node> {
    prev_i: u8,
    next_i: u8,
    head: usize,
    tail: usize,
    ph: PhantomData<T>,
}

impl<T: 'static + Node> LinkedList<T> {
    pub fn init(&mut self, prev_i: u8, next_i: u8, head: &'static mut T, tail: &'static mut T) {
        self.prev_i = prev_i;
        self.next_i = next_i;
        self.head = head as *const _ as usize;
        self.tail = tail as *const _ as usize;
        self.head().pointers_mut()[self.next_i as usize] = self.tail;
        self.tail().pointers_mut()[self.prev_i as usize] = self.head;
    }

    pub fn head(&self) -> &'static mut T {
        Self::cast(self.head)
    }

    pub fn tail(&self) -> &'static mut T {
        Self::cast(self.tail)
    }

    #[inline]
    fn cast(p: usize) -> &'static mut T {
        unsafe { &mut *(p as *mut _) }
    }

    pub fn is_empty(&self) -> bool {
        self.head().pointers()[self.next_i as usize] == self.tail
    }

    pub fn len(&self) -> usize {
        let mut cur = self.head;
        let mut i = 0;

        loop {
            let c = Self::cast(cur);
            if c.pointers()[self.next_i as usize] == self.tail {
                break;
            }
            cur = c.pointers()[self.next_i as usize];
            i += 1;
        }
        i
    }

    fn link_prev(&self, dst: &mut T, prev: &mut T) {
        dst.pointers_mut()[self.prev_i as usize] = prev as *const _ as usize;
    }

    fn link_next(&self, dst: &mut T, next: &mut T) {
        dst.pointers_mut()[self.next_i as usize] = next as *const _ as usize;
    }

    fn eq(l: &T, r: &T) -> bool {
        l as *const _ as usize == r as *const _ as usize
    }

    fn prev_of(&self, dst: &mut T) -> Option<&'static mut T> {
        dst.ref_at(self.prev_i as usize)
    }

    fn next_of(&self, dst: &mut T) -> Option<&'static mut T> {
        dst.ref_at(self.next_i as usize)
    }

    fn detach(&self, dst: &mut T) {
        let p = self.prev_of(dst).unwrap();
        let n = self.next_of(dst).unwrap();

        dst.pointers_mut()[self.prev_i as usize] = 0;
        dst.pointers_mut()[self.next_i as usize] = 0;

        self.link_next(p, n);
        self.link_prev(n, p);
    }

    pub fn pop_head(&mut self) -> Option<&'static mut T> {
        if self.is_empty() {
            None
        } else {
            let n = self.head().ref_at(self.next_i as usize).unwrap();
            self.detach(n);
            Some(n)
        }
    }

    pub fn prepend(&self, dst: &mut T, n: &mut T) {
        let prev = dst.ref_at(self.prev_i as usize).unwrap();
        self.link_next(prev, n);
        self.link_prev(n, prev);
        self.link_next(n, dst);
        self.link_prev(dst, n);
    }

    pub fn append(&mut self, n: &mut T) {
        assert!(n.pointers()[self.prev_i as usize] == 0 && n.pointers()[self.next_i as usize] == 0, "node already on list");
        let t = self.tail();
        self.prepend(t, n);
    }
}