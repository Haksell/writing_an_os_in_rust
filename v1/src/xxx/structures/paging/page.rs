pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
}
