use core::ops::Add;
use crate::println;
use crate::thread::reg::IntCtx;
use crate::vga::put_char;

pub fn sys_handle(ctx: &'static mut IntCtx) {
    use rlib::sys::NR;
    match ctx.eax {
        NR::WRITE => {
            let p: *const u8 = ctx.ebx as _;
            for i in 0..ctx.ecx {
                unsafe { put_char(*p.add(i as usize)) };
            }
        },
        _ => {}
    }
}


pub fn init() {
    crate::int::register(crate::int::SYS_VEC as u16, sys_handle);
}