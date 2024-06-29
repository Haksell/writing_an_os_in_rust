//! Representations of various x86 specific structures and descriptor tables.

use crate::xxx::VirtAddr;

pub mod gdt;
pub mod idt;
pub mod paging;
pub mod tss;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: VirtAddr,
}
