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
    #[inline]
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    #[inline]
    pub fn index(self) -> u16 {
        self.0 >> 3
    }

    #[inline]
    pub fn rpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
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

#[derive(Debug)]
pub struct CS;

#[derive(Debug)]
pub struct SS;

#[derive(Debug)]
pub struct DS;

#[derive(Debug)]
pub struct ES;

#[derive(Debug)]
pub struct FS;

#[derive(Debug)]
pub struct GS;
