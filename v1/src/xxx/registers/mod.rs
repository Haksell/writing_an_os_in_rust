pub mod rflags; // TODO: remove

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, rpl: u8) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }
}
