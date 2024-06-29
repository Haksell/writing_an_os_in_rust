use crate::xxx::registers::segmentation::{Segment, SegmentSelector, CS};
use core::arch::asm;

impl Segment for CS {
    #[inline]
    fn get_reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!(concat!("mov {0:x}, cs"), out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }
}
