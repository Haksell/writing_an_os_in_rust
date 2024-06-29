use crate::xxx::structures::paging::page::{PageSize, Size4KiB};
use crate::xxx::PhysAddr;
use core::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    pub(crate) start_address: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    #[inline]
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down_u64(S::SIZE),
            size: PhantomData,
        }
    }

    #[inline]
    pub fn start_address(self) -> PhysAddr {
        self.start_address
    }
}
