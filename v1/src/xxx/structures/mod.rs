use crate::xxx::VirtAddr;

pub mod gdt;
pub mod idt;
pub mod paging;
pub mod tss;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: VirtAddr,
}
