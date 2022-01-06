use crate::thread::sync::Lock;

pub fn init_locks() {
    unsafe {
        for x in [
            (&mut crate::vga::VGA_LOCK_REF, &mut crate::vga::VGA_LOCK),
            (&mut crate::mem::K_LOCK_REF, &mut crate::mem::K_LOCK),
            (&mut crate::mem::U_LOCK_REF, &mut crate::mem::U_LOCK),
        ] {
           *x.0 =  Lock::new(x.1.as_ptr() as usize, x.1.len()) as *const _ as usize;
        }
    }
}