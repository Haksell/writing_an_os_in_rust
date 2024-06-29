pub mod gdt;
pub mod idt;
pub mod tss;

use crate::virt_addr::VirtAddr;

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: VirtAddr,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, rpl: u8) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }
}
