use crate::xxx::VirtAddr;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    #[inline]
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down_u64(S::SIZE),
            size: PhantomData,
        }
    }

    #[inline]
    pub fn start_address(self) -> VirtAddr {
        self.start_address
    }
}

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PageRange<S: PageSize = Size4KiB> {
    pub start: Page<S>,
    pub end: Page<S>,
}

impl<S: PageSize> Iterator for PageRange<S> {
    type Item = Page<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let page = self.start;
            self.start += 1;
            Some(page)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PageRangeInclusive<S: PageSize = Size4KiB> {
    pub start: Page<S>,
    pub end: Page<S>,
}

impl<S: PageSize> Iterator for PageRangeInclusive<S> {
    type Item = Page<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let page = self.start;

            // If the end of the inclusive range is the maximum page possible for size S,
            // incrementing start until it is greater than the end will cause an integer overflow.
            // So instead, in that case we decrement end rather than incrementing start.
            let max_page_addr = VirtAddr::new(u64::MAX) - (S::SIZE - 1);
            if self.start.start_address() < max_page_addr {
                self.start += 1;
            } else {
                self.end -= 1;
            }
            Some(page)
        } else {
            None
        }
    }
}
