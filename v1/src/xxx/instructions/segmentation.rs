use crate::xxx::registers::segmentation::{Segment, SegmentSelector, CS};
use core::arch::asm;

macro_rules! segment_impl {
    ($type:ty, $name:literal) => {
        impl Segment for $type {
            #[inline]
            fn get_reg() -> SegmentSelector {
                let segment: u16;
                unsafe {
                    asm!(concat!("mov {0:x}, ", $name), out(reg) segment, options(nomem, nostack, preserves_flags));
                }
                SegmentSelector(segment)
            }
        }
    };
}

segment_impl!(CS, "cs");
