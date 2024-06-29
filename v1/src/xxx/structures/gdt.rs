//! Types for the Global Descriptor Table and segment selectors.

pub use crate::xxx::registers::segmentation::SegmentSelector;
use bitflags::bitflags;
use core::sync::atomic::{AtomicU64 as EntryValue, Ordering};

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
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct DescriptorFlags: u64 {
        const ACCESSED          = 1 << 40;
        const WRITABLE          = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const DPL_RING_3        = 3 << 45;
        const PRESENT           = 1 << 47;
        const AVAILABLE         = 1 << 52;
        const LONG_MODE         = 1 << 53;
        const DEFAULT_SIZE      = 1 << 54;
        const GRANULARITY       = 1 << 55;

        const LIMIT_0_15        = 0xFFFF;
        const LIMIT_16_19       = 0xF << 48;
        const BASE_0_23         = 0xFF_FFFF << 16;
        const BASE_24_31        = 0xFF << 56;
    }
}
