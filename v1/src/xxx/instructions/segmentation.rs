pub use crate::xxx::registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
use crate::xxx::structures::gdt::SegmentSelector;
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
segment_impl!(SS, "ss");
segment_impl!(DS, "ds");
segment_impl!(ES, "es");
segment_impl!(FS, "fs");
segment_impl!(GS, "gs");
