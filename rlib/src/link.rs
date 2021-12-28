pub trait Node: Sized {
    fn pointers_mut(&mut self) -> &mut [usize];
    fn pointers(&self) -> &[usize];

    fn ref_at(&self, i: usize) -> &'static mut Self {
        unsafe { &mut *(self.pointers()[i] as *mut Self) }
    }
}


pub struct LinkedList<T: 'static + Node> {
    prev_i: u8,
    next_i: u8,
    head: &'static mut T,
    tail: &'static mut T,
}

impl<T: 'static + Node> LinkedList<T> {
    pub fn init(&mut self, prev_i: u8, next_i: u8, head: &'static mut T, tail: &'static mut T) {
        self.prev_i = prev_i;
        self.next_i = next_i;
        self.head = head;
        self.tail = tail;
        self.head.pointers_mut()[self.next_i as usize] = self.tail as *mut _ as usize;
        self.tail.pointers_mut()[self.prev_i as usize] = self.head as *mut _ as usize;
    }

    pub fn is_empty(&self) -> bool {
        self.head.pointers()[self.next_i as usize] == self.tail as *const _ as usize
    }

    pub fn len(&self) -> usize {
        let mut cur = self.head as *const T;
        let mut i = 0;

        loop {
            let c = unsafe { &*cur };
            if c.pointers()[self.next_i as usize] == self.tail as *const _ as usize {
                break;
            }
            cur = c.ref_at(self.next_i as usize) as *const T;
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

    fn prev_of(&self, dst: &mut T) -> &'static mut T {
        dst.ref_at(self.prev_i as usize)
    }

    fn next_of(&self, dst: &mut T) -> &'static mut T {
        dst.ref_at(self.next_i as usize)
    }

    fn detach(&self, dst: &mut T) {}
}