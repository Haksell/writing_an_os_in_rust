mod gdt;
mod idt;
mod tss;

pub use gdt::{Gdt, GdtDescriptor};
pub use idt::{InterruptDescriptorTable, InterruptStackFrame};
pub use tss::TaskStateSegment;

use crate::virt_addr::VirtAddr;

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    limit: u16,
    base: VirtAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const fn new(index: u16, rpl: u16) -> Self {
        Self(index << 3 | rpl)
    }
}
