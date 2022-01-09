#![cfg_attr(not(test), no_std)]
#![feature(asm)]

use core::ops::{Add, Div, Sub};
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
#[cfg(target_arch = "x86")]
pub mod sys;

pub trait IntoRp {
    fn rs<T: Sized>(self) -> *const T;
}

impl IntoRp for usize {
    fn rs<T: Sized>(self) -> *const T {
        self as *const T
    }
}

pub trait Rp<T: Sized> {
   fn rf(self) -> &'static mut T;
    fn off(self) -> usize;
}


impl<T: Sized> Rp<T> for *const T {
    fn rf(self) -> &'static mut T {
        unsafe {
            &mut *(self as usize as *mut _)
        }
    }

    fn off(self) -> usize {
        self as usize
    }
}


#[macro_export]
macro_rules! div_up {
    ($e: expr, $div: expr) => {
        {
            ($e + $div - 1) / $div
        }
    };
}

#[macro_export]
macro_rules! size_of {
    ($t: ty) => {
        {
            { core::mem::size_of::<$t>() }
        }
    };
}

pub type Ref<T> = &'static mut T;