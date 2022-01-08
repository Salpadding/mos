use crate::println;
use crate::thread::reg::IntCtx;

pub fn sys_handle(ctx: &'static mut IntCtx) {
    ctx.eax = 127;
}


pub fn init() {
    crate::int::register(crate::int::SYS_VEC as u16, sys_handle);
}