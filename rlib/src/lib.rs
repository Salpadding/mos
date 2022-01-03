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

pub mod bitmap;
pub mod link;
pub mod gdt;

pub type Ref<T> = &'static mut T;