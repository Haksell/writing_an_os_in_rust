use crate::xxx::VirtAddr;
use core::fmt;
use core::iter::Step;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: u64;
    const DEBUG_STR: &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2MiB {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
    const DEBUG_STR: &'static str = "4KiB";
}

impl PageSize for Size2MiB {
    const SIZE: u64 = Size4KiB::SIZE * 512;
    const DEBUG_STR: &'static str = "2MiB";
}

impl PageSize for Size1GiB {
    const SIZE: u64 = Size2MiB::SIZE * 512;
    const DEBUG_STR: &'static str = "1GiB";
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

    pub(crate) fn steps_between_impl(start: &Self, end: &Self) -> Option<usize> {
        VirtAddr::steps_between_impl(&start.start_address, &end.start_address)
            .map(|steps| steps / S::SIZE as usize)
    }

    pub(crate) fn forward_checked_impl(start: Self, count: usize) -> Option<Self> {
        let count = count.checked_mul(S::SIZE as usize)?;
        let start_address = VirtAddr::forward_checked_impl(start.start_address, count)?;
        Some(Self {
            start_address,
            size: PhantomData,
        })
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

impl<S: PageSize> Sub<Self> for Page<S> {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

impl<S: PageSize> Step for Page<S> {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Self::steps_between_impl(start, end)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Self::forward_checked_impl(start, count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let count = count.checked_mul(S::SIZE as usize)?;
        let start_address = Step::backward_checked(start.start_address, count)?;
        Some(Self {
            start_address,
            size: PhantomData,
        })
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

#[derive(Debug)]
pub struct AddressNotAligned;

impl fmt::Display for AddressNotAligned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the given address was not sufficiently aligned")
    }
}
