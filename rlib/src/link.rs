use core::marker::PhantomData;

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

pub struct Iter<T: Node> {
    cur: usize,
    tail: usize,
    next_i: u8,
    ph: PhantomData<T>,
}

pub struct RawIter<T: Node> {
    cur: usize,
    tail: usize,
    next_i: u8,
    ph: PhantomData<T>,
}


impl<T: 'static + Node> Iterator for RawIter<T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.tail {
            None
        } else {
            let r = self.cur;
            let c: &'static mut T = LinkedList::cast(self.cur);
            self.cur = c.pointers()[self.next_i as usize];
            Some(r)
        }
    }
}

impl<T: 'static + Node> Iterator for Iter<T> {
    type Item = &'static mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.tail {
            None
        } else {
            let c: &'static mut T = LinkedList::cast(self.cur);
            self.cur = c.pointers()[self.next_i as usize];
            Some(c)
        }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct LinkedList<T: Node> {
    pub prev_i: u8,
    pub next_i: u8,
    pub head: usize,
    pub tail: usize,
    pub ph: PhantomData<T>,
}

impl<T: 'static + Node> LinkedList<T> {
    pub fn alloc_size() -> usize {
        8 * 4 + (core::mem::size_of::<T>() + 7) / 8 * 8 * 2
    }

    pub fn new(off: usize, prev_i: u8, next_i: u8) -> &'static mut Self {
        let t = unsafe { &mut *(off as *mut Self) };
        let head = off + 8 * 4;
        let tail = head + (core::mem::size_of::<T>() + 7) / 8 * 8;
        t.init(prev_i, next_i, Self::cast(head), Self::cast(tail));
        t
    }

    pub fn init(&mut self, prev_i: u8, next_i: u8, head: &'static mut T, tail: &'static mut T) {
        self.prev_i = prev_i;
        self.next_i = next_i;
        self.head = head as *const _ as usize;
        self.tail = tail as *const _ as usize;
        self.head().pointers_mut()[self.next_i as usize] = self.tail;
        self.tail().pointers_mut()[self.prev_i as usize] = self.head;
        self.ph = PhantomData::default();
    }

    pub fn iter(&self) -> Iter<T> {
        let cur = self.head().pointers()[self.next_i as usize];
        Iter {
            cur,
            tail: self.tail,
            next_i: self.next_i,
            ph: Default::default(),
        }
    }

    pub fn raw_iter(&self) -> RawIter<T> {
        let cur = self.head().pointers()[self.next_i as usize];
        RawIter {
            cur,
            tail: self.tail,
            next_i: self.next_i,
            ph: Default::default(),
        }
    }

    fn head(&self) -> &'static mut T {
        Self::cast(self.head)
    }

    fn tail(&self) -> &'static mut T {
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
        self.iter().count()
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

    fn prepend(&self, dst: &mut T, n: &mut T) {
        let prev = dst.ref_at(self.prev_i as usize).unwrap();
        self.link_next(prev, n);
        self.link_prev(n, prev);
        self.link_next(n, dst);
        self.link_prev(dst, n);
    }

    #[inline]
    fn assert_not_contains(&self, n: &mut T) {
        assert!(n.pointers()[self.prev_i as usize] == 0 && n.pointers()[self.next_i as usize] == 0, "node already on list");
    }

    pub fn append(&mut self, n: &mut T) {
        self.assert_not_contains(n);
        let t = self.tail();
        self.prepend(t, n);
    }

    pub fn first(&self) -> Option<&'static mut T> {
        if self.is_empty() { None } else { self.head().ref_at(self.next_i as usize) }
    }

    pub fn push_head(&mut self, n: &mut T) {
        self.assert_not_contains(n);

        let f = self.first();

        match f {
            Some(x) => self.prepend(x, n),
            _ => self.prepend(self.tail(), n)
        }
    }
}