//! Abstractions for default-sized and huge virtual memory pages.

use crate::xxx::sealed::Sealed;
use crate::xxx::structures::paging::PageTableIndex;
use crate::xxx::VirtAddr;
use core::fmt;
use core::iter::Step;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Trait for abstracting over the three possible page sizes on x86_64, 4KiB, 2MiB, 1GiB.
pub trait PageSize: Copy + Eq + PartialOrd + Ord + Sealed {
    /// The page size in bytes.
    const SIZE: u64;

    /// A string representation of the page size for debug output.
    const DEBUG_STR: &'static str;
}

/// This trait is implemented for 4KiB and 2MiB pages, but not for 1GiB pages.
pub trait NotGiantPageSize: PageSize {}

/// A standard 4KiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

/// A “huge” 2MiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2MiB {}

/// A “giant” 1GiB page.
///
/// (Only available on newer x86_64 CPUs.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
    const DEBUG_STR: &'static str = "4KiB";
}

impl NotGiantPageSize for Size4KiB {}

impl Sealed for super::Size4KiB {}

impl PageSize for Size2MiB {
    const SIZE: u64 = Size4KiB::SIZE * 512;
    const DEBUG_STR: &'static str = "2MiB";
}

impl NotGiantPageSize for Size2MiB {}

impl Sealed for super::Size2MiB {}

impl PageSize for Size1GiB {
    const SIZE: u64 = Size2MiB::SIZE * 512;
    const DEBUG_STR: &'static str = "1GiB";
}

impl Sealed for super::Size1GiB {}

/// A virtual memory page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// Returns the page that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid page start).
    #[inline]
    pub fn from_start_address(address: VirtAddr) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned_u64(S::SIZE) {
            return Err(AddressNotAligned);
        }
        Ok(Page::containing_address(address))
    }

    /// Returns the page that contains the given virtual address.
    #[inline]
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down_u64(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the page.
    #[inline]
    pub fn start_address(self) -> VirtAddr {
        self.start_address
    }

    /// Returns the level 4 page table index of this page.
    #[inline]
    pub fn p4_index(self) -> PageTableIndex {
        self.start_address().p4_index()
    }

    /// Returns the level 3 page table index of this page.
    #[inline]
    pub fn p3_index(self) -> PageTableIndex {
        self.start_address().p3_index()
    }

    /// Returns a range of pages, inclusive `end`.
    #[inline]
    pub fn range_inclusive(start: Self, end: Self) -> PageRangeInclusive<S> {
        PageRangeInclusive { start, end }
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

impl<S: NotGiantPageSize> Page<S> {
    /// Returns the level 2 page table index of this page.
    #[inline]
    pub fn p2_index(self) -> PageTableIndex {
        self.start_address().p2_index()
    }
}

impl Page<Size4KiB> {
    /// Returns the level 1 page table index of this page.
    #[inline]
    pub const fn p1_index(self) -> PageTableIndex {
        self.start_address.p1_index()
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

/// A range of pages with exclusive upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PageRange<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: Page<S>,
    /// The end of the range, exclusive.
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

/// A range of pages with inclusive upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PageRangeInclusive<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: Page<S>,
    /// The end of the range, inclusive.
    pub end: Page<S>,
}

impl<S: PageSize> PageRangeInclusive<S> {
    /// Returns whether this range contains no pages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }
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

/// The given address was not sufficiently aligned.
#[derive(Debug)]
pub struct AddressNotAligned;

impl fmt::Display for AddressNotAligned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the given address was not sufficiently aligned")
    }
}
