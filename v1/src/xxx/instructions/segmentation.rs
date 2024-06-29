pub use crate::xxx::registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
use crate::xxx::structures::gdt::SegmentSelector;
use core::arch::asm;

macro_rules! get_reg_impl {
    ($name:literal) => {
        #[inline]
        fn get_reg() -> SegmentSelector {
            let segment: u16;
            unsafe {
                asm!(concat!("mov {0:x}, ", $name), out(reg) segment, options(nomem, nostack, preserves_flags));
            }
            SegmentSelector(segment)
        }
    };
}

macro_rules! segment_impl {
    ($type:ty, $name:literal) => {
        impl Segment for $type {
            get_reg_impl!($name);
        }
    };
}

impl Segment for CS {
    get_reg_impl!("cs");
}

segment_impl!(SS, "ss");
segment_impl!(DS, "ds");
segment_impl!(ES, "es");
segment_impl!(FS, "fs");
segment_impl!(GS, "gs");
