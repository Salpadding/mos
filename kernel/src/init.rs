use crate::thread::sync::Lock;

pub fn init_locks() {
    unsafe {
        crate::vga::VGA_LOCK_REF = Lock::new(crate::vga::VGA_LOCK.as_ptr() as usize, crate::vga::VGA_LOCK.len()) as *const _ as usize;
    }
}