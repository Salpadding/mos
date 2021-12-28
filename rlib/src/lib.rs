#![cfg_attr(not(test), no_std)]

#[macro_export]
macro_rules! alloc_static {
    ($var: ident, $f: ident, $t: ty) => {
        static mut $var: [u8; core::mem::size_of::<$t>()] = [0u8; core::mem::size_of::<$t>()];

        pub fn $f() -> &'static mut $t {
            unsafe { &mut *($var.as_ptr() as *mut $t) }
        }
    };
}

#[macro_export]
macro_rules! alloc_statics {
    ($var: ident, $f: ident, $t: ty, $cnt: expr) => {
        static mut $var: [u8; core::mem::size_of::<$t>() * $cnt] = [0u8; core::mem::size_of::<$t>() * $cnt];

        pub fn $f() -> &'static mut [$t<'static>] {
            unsafe {
                core::slice::from_raw_parts_mut(
                    $var.as_ptr() as *mut _,
                    $cnt
                )
            }
        }
    };
}

pub mod bitmap;
pub mod list;
mod link;

pub type Ref<T> = &'static mut T;