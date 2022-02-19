use rlib::{alloc_static, as_str, div_up};
use rlib::args::SliceWriter;
use rlib::link::{LinkedList, Node};

use crate::thread::current_pcb;
use crate::{c_print, c_println, println, sleep_mils};
use crate::asm::out_b;
use crate::err::SE;
use crate::fs::ctl::{BIT_DEV_DEV, BIT_DEV_LBA, BIT_DEV_MBS, BIT_STAT_BSY, BIT_STAT_DRQ, CMD_ID, CMD_READ_SEC};
use crate::fs::DiskInfo;
use crate::int::register;
use crate::thread::reg::IntCtx;
use crate::thread::sync::{Lock, Semaphore};

const DEBUG: bool = true;
const IDE_CHANNELS: usize = 2;
const DISKS: usize = 2;
const NAME_BUF_LEN: usize = 8;
pub const PRIMARY_LEN: usize = 4;
const LOGICAL_LEN: usize = 8;
const BUSY_WAITING_MILS: u32 = 30 * 1000;
const SEC_SIZE: usize = 512;
const MAX_LBA: u32 = 80 * 1024 * 1024 / 512 - 1;

alloc_static!(PARTITION_LIST, partitions, LinkedList<Partition, 64>);
alloc_static!(IDE_CHANNEL, channels, [IdeChannel; IDE_CHANNELS]);

pub struct Partition {
    pointers: [usize; 2],
    start: u32,
    // start sector, lba
    sec_n: u32,
    // the number of sectors belongs to this partition
    disk: usize,
    // the disk where this partition belongs to
    name: [u8; NAME_BUF_LEN],
    // name of this partition
    sb: usize,
    // super block of this partition
    blocks: Option<&'static mut [u8]>,
    // bitmap of blocks
    inodes: Option<&'static mut [u8]>, // inode bitmaps
}

impl Node for Partition {
    fn pointers_mut(&mut self) -> &mut [usize] {
        &mut self.pointers
    }

    fn pointers(&self) -> &[usize] {
        &self.pointers
    }
}

pub struct Disk {
    pub name: [u8; NAME_BUF_LEN],
    // name of this disk
    pub ide: usize,
    // the channel where this disk belongs to, primary -> 1, slave -> 0
    pub dev_no: u8,
    pub primary_parts: [Option<Partition>; PRIMARY_LEN],
    pub logical_parts: [Option<Partition>; LOGICAL_LEN],
}

impl Disk {
    pub fn ide(&self) -> &'static mut IdeChannel {
        cst!(self.ide)
    }

    pub fn select(&self) {
        let mut dev = BIT_DEV_MBS | BIT_DEV_LBA;
        dev |= if self.dev_no > 0 { BIT_DEV_DEV } else { 0 };
        let ch = self.ide();
        out_b(ch.reg_dev(), dev);
    }

    pub fn name(&self) -> &str {
        as_str(&self.name)
    }
    // get identity of the disk
    pub fn init(&mut self) {
        let ch = self.ide();
        self.select();
        ch.cmd_out(CMD_ID);

        // block current thread until disk ready
        c_println!("ch {} disk_done.p()", ch.name());
        ch.disk_done.p();

        if !self.busy_wait(BUSY_WAITING_MILS) {
            panic!("wait on {} failed", self.name());
        }

        let mut buf: [u8; SEC_SIZE] = [0u8; SEC_SIZE];
        // read a section from disk
        self.read_secs(&mut buf, 1);

        let mut w = crate::asm::Writer {};
        use super::DiskInfo;
        c_print!("sn = ");
        buf.write_sn(&mut w);
        c_print!("module = ");
        buf.write_md(&mut w);

        c_println!("sectors = {}", buf.sectors());
        c_println!("cap = {} MB", buf.sectors() * 512 / 1024 / 1024);
    }

    // wait until disk ready
    pub fn busy_wait(&self, mut mils: u32) -> bool {
        let ch = self.ide();
        let st_p = ch.reg_status();

        while mils > 0 {
            if crate::asm::in_b(st_p) & BIT_STAT_BSY == 0 {
                return crate::asm::in_b(st_p) & BIT_STAT_DRQ != 0;
            }
            sleep_mils(10);
            mils -= 10;
        }
        false
    }

    // read n sections info buffer, require buf.len() >= sec_n * 512
    pub fn read_secs(&self, buf: &mut [u8], sec_n: u8) {
        let bytes = if sec_n == 0 { 256 * SEC_SIZE } else { sec_n as usize * SEC_SIZE };
        let ch = self.ide();

        // convert buf to u16
        assert!(buf.len() >= bytes, "size of buf {} < bytes {}", buf.len(), bytes);
        let b = unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u16, bytes / 2) };
        crate::asm::in_sw(ch.reg_data(), b);
    }

    pub fn ide_read(&self, lba: u32, buf: &mut [u8], sec_n: usize) {
        c_println!("ide read lba = {}, sec_n = {}", lba, sec_n);
        assert!(lba < MAX_LBA, "lba {} overflow", lba);
        assert!(buf.len() >= sec_n * SEC_SIZE, "buf.len() {} < sec_bytes {}", buf.len(), sec_n * SEC_SIZE);

        let ch = self.ide();

        let gd = ch.lock.lock();
        c_println!("guard get");
        self.select();

        let mut dones: usize = 0;

        while dones < sec_n {
            // get at most 256 sectors every command
            let todo = (sec_n - dones).min(256);

            self.select_sec(lba + dones as u32, todo as u8);
            ch.cmd_out(CMD_READ_SEC);
            c_println!("ch {} done p()", ch.name());
            ch.disk_done.p();
            c_println!("return from ch {} done p()", ch.name());

            if !self.busy_wait(BUSY_WAITING_MILS) {
                panic!("busy wait failed for device {}", self.name());
            }

            // read into buffer
            self.read_secs(&mut buf[dones * SEC_SIZE..], todo as u8);
            dones += todo;
            c_println!("dones = {} now ", dones);
        }

        c_println!("read done");
    }

    fn select_sec(&self, lba: u32, sec_n: u8) {
        let ch = self.ide();

        // read 256 sector if sec_n = 0
        out_b(ch.reg_sec_n(), sec_n);
        ch.out_lba(self.dev_no, lba);
    }

    fn part_scan(&mut self) {
        for x in self.primary_parts.iter_mut() {
            *x = None;
        }
        for x in self.logical_parts.iter_mut() {
            *x = None;
        }
        let mut boot: [u8; SEC_SIZE] = [0u8; SEC_SIZE];
        self.ide_read(0, &mut boot, 1);
    }
}

pub struct IdeChannel {
    pub name: [u8; NAME_BUF_LEN],
    // name of ata channel
    pub port: u16,
    // start port
    pub int_vec: u16,
    //
    pub lock: Lock,
    // lock of ide channel
    pub expecting: bool,
    pub disk_done: Semaphore,
    pub devices: [Disk; DISKS],
}

impl IdeChannel {
    pub fn name(&self) -> &str {
        rlib::as_str(&self.name)
    }

    pub fn reg_dev(&self) -> u16 {
        self.port + 6
    }

    pub fn reg_cmd(&self) -> u16 {
        self.port + 7
    }

    pub fn reg_status(&self) -> u16 {
        self.port + 7
    }

    pub fn reg_data(&self) -> u16 {
        self.port
    }

    pub fn reg_sec_n(&self) -> u16 {
        self.port + 2
    }

    #[inline]
    pub fn out_lba(&self, dev_no: u8, lba: u32) {
        out_b(self.port + 3, lba as u8);
        out_b(self.port + 4, (lba >> 8) as u8);
        out_b(self.port + 5, (lba >> 16) as u8);
        out_b(self.port + 6,
              // write lba 24~27 to 0~3 of
              BIT_DEV_MBS | BIT_DEV_LBA | if dev_no > 0 { BIT_DEV_DEV } else { 0 } |
                  (lba >> 24 & 0xf) as u8,
        );
    }

    // send command to the channel
    pub fn cmd_out(&mut self, cmd: u8) {
        self.expecting = true;
        out_b(self.reg_cmd(), cmd);
    }
}


pub fn init() {
    // initialize linked list
    let parts = partitions();
    parts.init(0, 1);

    if DEBUG {
        c_println!("parts initialized");
    }

    let ch_cnt = div_up!(crate::fs::disks() as usize, 2);

    if DEBUG {
        c_println!("ch_cnt = {}", ch_cnt);
    }

    let chs = channels();

    for ch_no in 0..ch_cnt {
        use core::fmt::Write;

        let ch = &mut chs[ch_no];

        if DEBUG {
            c_println!("before format args");
        }

        let mut sw = SliceWriter::new(&mut ch.name);
        write!(sw, "ide-{}", ch_no);

        if DEBUG {
            c_println!("after format args");
        }

        if ch_no == 0 {
            ch.port = 0x1f0;
            ch.int_vec = 0x20 + 14;
        } else {
            ch.port = 0x170;
            ch.int_vec = 0x20 + 15;
        }

        ch.expecting = false;

        // initialize lock
        ch.lock.init();

        // initialize the value as zero, call v() in interrupt handler
        ch.disk_done.value = 0;
        ch.disk_done.waiters.init(2, 3);

        register(ch.int_vec, int_handle);

        // get parameters of two disks and partition info
        for dev_no in 0..DISKS {
            let ch_p = ch as *const _ as usize;
            let hd = &mut ch.devices[dev_no];
            hd.dev_no = dev_no as u8;
            hd.ide = ch_p;
            let mut sw = SliceWriter::new(&mut hd.name);
            write!(sw, "sd{}", (b'a' + ch_no as u8 * 2 + dev_no as u8) as char);
            c_println!("hd = {}", hd.name());

            hd.init();

            c_println!("hd {} init() success", hd.name());

            if dev_no != 0 {
                c_println!("before hd {} part scan", hd.name());
                hd.part_scan();
                c_println!("hd {} part_scan() success", hd.name());
            }
        }
    }
}


pub fn int_handle(ctx: &'static mut IntCtx) {
    assert!(ctx.vec == 0x2e || ctx.vec == 0x2f, "ide::int_handle(): invalid vec");
    c_println!("ide::int_handle: {}", ctx.vec);
    let ch_no = ctx.vec - 0x2e;
    let chs = channels();
    let ch = &mut chs[ch_no as usize];
    assert_eq!(ch.int_vec, ctx.vec as u16, "int vec");

    if !ch.expecting {
        return;
    }

    ch.expecting = false;
    c_println!("ch {} disk_done v()", ch.name());
    c_println!("waiters = {}", ch.disk_done.waiters.first().map(|x| x.name()).unwrap_or(""));
    ch.disk_done.v();
    crate::asm::in_b(ch.reg_status());
}