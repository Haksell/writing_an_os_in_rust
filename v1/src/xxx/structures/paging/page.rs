use crate::xxx::addr::VirtAddr;
use core::marker::PhantomData;

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}
