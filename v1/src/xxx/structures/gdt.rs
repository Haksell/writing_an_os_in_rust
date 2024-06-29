//! Types for the Global Descriptor Table and segment selectors.

pub use crate::xxx::registers::segmentation::SegmentSelector;
use bitflags::bitflags;
use core::sync::atomic::{AtomicU64 as EntryValue, Ordering};

/// 8-byte entry in a descriptor table.
///
/// A [`GlobalDescriptorTable`] (or LDT) is an array of these entries, and
/// [`SegmentSelector`]s index into this array. Each [`Descriptor`] in the table
/// uses either 1 Entry (if it is a [`UserSegment`](Descriptor::UserSegment)) or
/// 2 Entries (if it is a [`SystemSegment`](Descriptor::SystemSegment)). This
/// type exists to give users access to the raw entry bits in a GDT.
#[repr(transparent)]
pub struct Entry(EntryValue);

impl Entry {
    const fn new(raw: u64) -> Self {
        Self(EntryValue::new(raw))
    }

    pub fn raw(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self::new(self.raw())
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for Entry {}

bitflags! {
    /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct DescriptorFlags: u64 {
        /// Set by the processor if this segment has been accessed. Only cleared by software.
        /// _Setting_ this bit in software prevents GDT writes on first use.
        const ACCESSED          = 1 << 40;
        /// For 32-bit data segments, sets the segment as writable. For 32-bit code segments,
        /// sets the segment as _readable_. In 64-bit mode, ignored for all segments.
        const WRITABLE          = 1 << 41;
        /// For code segments, sets the segment as “conforming”, influencing the
        /// privilege checks that occur on control transfers. For 32-bit data segments,
        /// sets the segment as "expand down". In 64-bit mode, ignored for data segments.
        const CONFORMING        = 1 << 42;
        /// This flag must be set for code segments and unset for data segments.
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// These two bits encode the Descriptor Privilege Level (DPL) for this descriptor.
        /// If both bits are set, the DPL is Ring 3, if both are unset, the DPL is Ring 0.
        const DPL_RING_3        = 3 << 45;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Available for use by the Operating System
        const AVAILABLE         = 1 << 52;
        /// Must be set for 64-bit code segments, unset otherwise.
        const LONG_MODE         = 1 << 53;
        /// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][Self::LONG_MODE] is set,
        /// this must be unset. In 64-bit mode, ignored for data segments.
        const DEFAULT_SIZE      = 1 << 54;
        /// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all segments.
        const GRANULARITY       = 1 << 55;

        /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
        const LIMIT_0_15        = 0xFFFF;
        /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
        const LIMIT_16_19       = 0xF << 48;
        /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_0_23         = 0xFF_FFFF << 16;
        /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_24_31        = 0xFF << 56;
    }
}
