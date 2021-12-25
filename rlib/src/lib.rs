#![cfg_attr(not(test), no_std)]

pub mod bitmap;
pub mod list;

pub type Ref<T> = &'static mut T;