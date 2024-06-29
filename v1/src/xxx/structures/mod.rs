use crate::xxx::virt_addr::VirtAddr;

pub mod idt;
pub mod tss;

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: VirtAddr,
}
