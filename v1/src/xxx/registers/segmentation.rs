use crate::xxx::PrivilegeLevel;
use bit_field::BitField;
use core::fmt;

pub trait Segment {
    fn get_reg() -> SegmentSelector;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// Creates a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index`: index in GDT or LDT array (not the offset)
    ///  * `rpl`: the requested privilege level
    #[inline]
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    /// Can be used as a selector into a non-existent segment and assigned to segment registers,
    /// e.g. data segment register in ring 0
    pub const NULL: Self = Self::new(0, PrivilegeLevel::Ring0);

    /// Returns the GDT index.
    #[inline]
    pub fn index(self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level.
    #[inline]
    pub fn rpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
    }

    /// Set the privilege level for this Segment selector.
    #[inline]
    pub fn set_rpl(&mut self, rpl: PrivilegeLevel) {
        self.0.set_bits(0..2, rpl as u16);
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("SegmentSelector");
        s.field("index", &self.index());
        s.field("rpl", &self.rpl());
        s.finish()
    }
}

/// Code Segment
///
/// While most fields in the Code-Segment [`Descriptor`] are unused in 64-bit
/// long mode, some of them must be set to a specific value. The
/// [`EXECUTABLE`](DescriptorFlags::EXECUTABLE),
/// [`USER_SEGMENT`](DescriptorFlags::USER_SEGMENT), and
/// [`LONG_MODE`](DescriptorFlags::LONG_MODE) bits must be set, while the
/// [`DEFAULT_SIZE`](DescriptorFlags::DEFAULT_SIZE) bit must be unset.
///
/// The [`DPL_RING_3`](DescriptorFlags::DPL_RING_3) field can be used to change
/// privilege level. The [`PRESENT`](DescriptorFlags::PRESENT) bit can be used
/// to make a segment present or not present.
///
/// All other fields (like the segment base and limit) are ignored by the
/// processor and setting them has no effect.
#[derive(Debug)]
pub struct CS;

/// Stack Segment
///
/// Entirely unused in 64-bit mode; setting the segment register does nothing.
/// However, in ring 3, the SS register still has to point to a valid
/// [`Descriptor`] (it cannot be zero). This
/// means a user-mode read/write segment descriptor must be present in the GDT.
///
/// This register is also set by the `syscall`/`sysret` and
/// `sysenter`/`sysexit` instructions (even on 64-bit transitions). This is to
/// maintain symmetry with 32-bit transitions where setting SS actually will
/// actually have an effect.
#[derive(Debug)]
pub struct SS;

/// Data Segment
///
/// Entirely unused in 64-bit mode; setting the segment register does nothing.
#[derive(Debug)]
pub struct DS;

/// ES Segment
///
/// Entirely unused in 64-bit mode; setting the segment register does nothing.
#[derive(Debug)]
pub struct ES;

/// FS Segment
///
/// Only base is used in 64-bit mode, see [`Segment64`]. This is often used in
/// user-mode for Thread-Local Storage (TLS).
#[derive(Debug)]
pub struct FS;

/// GS Segment
///
/// Only base is used in 64-bit mode, see [`Segment64`]. In kernel-mode, the GS
/// base often points to a per-cpu kernel data structure.
#[derive(Debug)]
pub struct GS;
