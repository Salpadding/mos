pub fn gdt(off: usize, len: usize) -> &'static mut [u64] {
    unsafe { core::slice::from_raw_parts_mut(off as *mut _, len) }
}

pub fn user_code() -> u64 {
    let mut bd = GdtBuilder::default();
    bd.limit(0xffffffff)
        .present(true)
        .executable(true)
        .rw(false)
        .mode(Mode::Protect)
        .privilege(3)
        .lim_4k(true)
        .system(false)
        .conforming(true)
        .build()
}

pub fn user_data() -> u64 {
    let mut bd = GdtBuilder::default();
    bd.limit(0xffffffff)
        .present(true)
        .executable(false)
        .rw(true)
        .mode(Mode::Protect)
        .privilege(3)
        .lim_4k(true)
        .system(false)
        .build()
}

pub enum Mode {
    Real,
    Protect,
    Long,
}

#[derive(Default)]
pub struct GdtBuilder {
    limit: u32,
    base: u32,
    access: u8,
    flags: u8,
}

impl GdtBuilder {
    pub fn limit(&mut self, lim: u32) -> &mut Self {
        self.limit = lim;
        self
    }

    pub fn base(&mut self, base: u32) -> &mut Self {
        self.base = base;
        self
    }

    pub fn access(&mut self, v: bool) -> &mut Self {
        if v {
            self.access |= 1;
        } else {
            self.access &= !1;
        }
        self
    }

    pub fn present(&mut self, v: bool) -> &mut Self {
        if v {
            self.access |= 1 << 7;
        } else {
            self.access &= !(1 << 7);
        }
        self
    }

    pub fn grow_down(&mut self, v: bool) -> &mut Self {
        // assert is data segment
        assert_eq!(self.access & 1 << 3, 0);

        if v {
            self.access |= 1 << 2;
        } else {
            self.access &= !(1 << 2);
        }
        self
    }

    pub fn conforming(&mut self, v: bool) -> &mut Self {
        // assert is code segment
        assert_ne!(self.access & 1 << 3, 0);

        if v {
            self.access |= 1 << 2;
        } else {
            self.access &= !(1 << 2);
        }
        self
    }

    pub fn rw(&mut self, v: bool) -> &mut Self {
        if v {
            self.access |= 1 << 1;
        } else {
            self.access &= !(1 << 1);
        }
        self
    }

    pub fn executable(&mut self, v: bool) -> &mut Self {
        if v {
            self.access |= 1 << 3;
        } else {
            self.access &= !(1 << 3);
        }
        self
    }

    pub fn privilege(&mut self, v: u8) -> &mut Self {
        assert!(v < 4);
        self.access |= v << 5;
        self
    }

    pub fn system(&mut self, v: bool) -> &mut Self {
        if v {
            self.access &= !(1 << 4);
        } else {
            self.access |= 1 << 4;
        }
        self
    }

    pub fn mode(&mut self, m: Mode) -> &mut Self {
        match m {
            Mode::Real => {
                self.flags &= !(1 << 2);
                self.flags &= !(1 << 1);
            }
            Mode::Protect => {
                self.flags |= 1 << 2;
                self.flags &= !(1 << 1);
            }
            Mode::Long => {
                self.flags &= !(1 << 2);
                self.flags |= 1 << 1;
            }
        }
        self
    }

    pub fn lim_4k(&mut self, v: bool) -> &mut Self {
        if v {
            self.flags |= 1 << 3;
        } else {
            self.flags &= !(1 << 3);
        }
        self
    }

    pub fn build(&self) -> u64 {
        let mut data: [u16; 4] = [0u16; 4];
        let mut buf: [u8; 8] = [0; 8];

        data[0] = (self.limit & 0xffff) as u16;
        data[1] = (self.base & 0xffff) as u16;
        data[2] |= ((self.base & 0xff0000) >> 16) as u16;
        data[2] |= (self.access as u16) << 8;
        data[3] |= ((self.limit & 0xf0000) >> 16) as u16;
        data[3] |= (self.flags as u16) << 4;
        data[3] |= (((self.base & 0xff000000) >> 24) << 8) as u16;

        for i in 0..data.len() {
            let tmp = data[i].to_le_bytes();
            buf[i * 2..i * 2 + 2].copy_from_slice(&tmp);
        }
        u64::from_le_bytes(buf)
    }
}
