pub mod gdt;
pub mod idt;
pub mod tss;

use crate::virt_addr::VirtAddr;

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    limit: u16,
    base: VirtAddr,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const fn new(index: u16, rpl: u16) -> Self {
        Self(index << 3 | rpl)
    }
}
