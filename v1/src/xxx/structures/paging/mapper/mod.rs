//! Abstractions for reading and modifying the mapping of pages.

pub use self::mapped_page_table::{MappedPageTable, PageTableFrameMapping};
#[cfg(target_pointer_width = "64")]
pub use self::offset_page_table::OffsetPageTable;
#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
pub use self::recursive_page_table::{InvalidPageTable, RecursivePageTable};

use crate::xxx::structures::paging::{
    frame_alloc::{FrameAllocator, FrameDeallocator},
    page::PageRangeInclusive,
    page_table::PageTableFlags,
    Page, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
};
use crate::xxx::{PhysAddr, VirtAddr};

mod mapped_page_table;
mod offset_page_table;
#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
mod recursive_page_table;

/// An empty convencience trait that requires the `Mapper` trait for all page sizes.
pub trait MapperAllSizes: Mapper<Size4KiB> + Mapper<Size2MiB> + Mapper<Size1GiB> {}

impl<T> MapperAllSizes for T where T: Mapper<Size4KiB> + Mapper<Size2MiB> + Mapper<Size1GiB> {}

/// Provides methods for translating virtual addresses.
pub trait Translate {
    /// Return the frame that the given virtual address is mapped to and the offset within that
    /// frame.
    ///
    /// If the given address has a valid mapping, the mapped frame and the offset within that
    /// frame is returned. Otherwise an error value is returned.
    ///
    /// This function works with huge pages of all sizes.
    fn translate(&self, addr: VirtAddr) -> TranslateResult;

    /// Translates the given virtual address to the physical address that it maps to.
    ///
    /// Returns `None` if there is no valid mapping for the given address.
    ///
    /// This is a convenience method. For more information about a mapping see the
    /// [`translate`](Translate::translate) method.
    #[inline]
    fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        match self.translate(addr) {
            TranslateResult::NotMapped | TranslateResult::InvalidFrameAddress(_) => None,
            TranslateResult::Mapped { frame, offset, .. } => Some(frame.start_address() + offset),
        }
    }
}

/// The return value of the [`Translate::translate`] function.
///
/// If the given address has a valid mapping, a `Frame4KiB`, `Frame2MiB`, or `Frame1GiB` variant
/// is returned, depending on the size of the mapped page. The remaining variants indicate errors.
#[derive(Debug)]
pub enum TranslateResult {
    /// The virtual address is mapped to a physical frame.
    Mapped {
        /// The mapped frame.
        frame: MappedFrame,
        /// The offset within the mapped frame.
        offset: u64,
        /// The entry flags in the lowest-level page table.
        ///
        /// Flags of higher-level page table entries are not included here, but they can still
        /// affect the effective flags for an address, for example when the WRITABLE flag is not
        /// set for a level 3 entry.
        flags: PageTableFlags,
    },
    /// The given virtual address is not mapped to a physical frame.
    NotMapped,
    /// The page table entry for the given virtual address points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

/// Represents a physical frame mapped in a page table.
#[derive(Debug)]
pub enum MappedFrame {
    /// The virtual address is mapped to a 4KiB frame.
    Size4KiB(PhysFrame<Size4KiB>),
    /// The virtual address is mapped to a "large" 2MiB frame.
    Size2MiB(PhysFrame<Size2MiB>),
    /// The virtual address is mapped to a "huge" 1GiB frame.
    Size1GiB(PhysFrame<Size1GiB>),
}

impl MappedFrame {
    /// Returns the start address of the frame.
    pub const fn start_address(&self) -> PhysAddr {
        match self {
            MappedFrame::Size4KiB(frame) => frame.start_address,
            MappedFrame::Size2MiB(frame) => frame.start_address,
            MappedFrame::Size1GiB(frame) => frame.start_address,
        }
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    pub const fn size(&self) -> u64 {
        match self {
            MappedFrame::Size4KiB(_) => Size4KiB::SIZE,
            MappedFrame::Size2MiB(_) => Size2MiB::SIZE,
            MappedFrame::Size1GiB(_) => Size1GiB::SIZE,
        }
    }
}

/// A trait for common page table operations on pages of size `S`.
pub trait Mapper<S: PageSize> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let parent_table_flags = flags
            & (PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE);

        unsafe {
            self.map_to_with_table_flags(page, frame, flags, parent_table_flags, frame_allocator)
        }
    }

    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized;

    /// Removes a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Note that no page tables or pages are deallocated.
    fn unmap(&mut self, page: Page<S>) -> Result<(PhysFrame<S>, MapperFlush<S>), UnmapError>;

    /// Updates the flags of an existing mapping.
    ///
    /// To read the current flags of a mapped page, use the [`Translate::translate`] method.
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn update_flags(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S>, FlagUpdateError>;

    /// Set the flags of an existing page level 4 table entry
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Set the flags of an existing page table level 3 entry
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Set the flags of an existing page table level 2 entry
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Return the frame that the specified page is mapped to.
    ///
    /// This function assumes that the page is mapped to a frame of size `S` and returns an
    /// error otherwise.
    fn translate_page(&self, page: Page<S>) -> Result<PhysFrame<S>, TranslateError>;

    /// Maps the given frame to the virtual page with the same address.
    ///
    /// ## Safety
    ///
    /// This is a convencience function that invokes [`Mapper::map_to`] internally, so
    /// all safety requirements of it also apply for this function.
    #[inline]
    unsafe fn identity_map<A>(
        &mut self,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
        S: PageSize,
        Self: Mapper<S>,
    {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        unsafe { self.map_to(page, frame, flags, frame_allocator) }
    }
}

/// This type represents a page whose mapping has changed in the page table.
///
/// The old mapping might be still cached in the translation lookaside buffer (TLB), so it needs
/// to be flushed from the TLB before it's accessed. This type is returned from a function that
/// changed the mapping of a page to ensure that the TLB flush is not forgotten.
#[derive(Debug)]
#[must_use = "Page Table changes must be flushed or ignored."]
#[cfg_attr(
    not(all(feature = "instructions", target_arch = "x86_64")),
    allow(dead_code)
)] // FIXME
pub struct MapperFlush<S: PageSize>(Page<S>);

impl<S: PageSize> MapperFlush<S> {
    /// Create a new flush promise
    ///
    /// Note that this method is intended for implementing the [`Mapper`] trait and no other uses
    /// are expected.
    #[inline]
    pub fn new(page: Page<S>) -> Self {
        MapperFlush(page)
    }

    /// Flush the page from the TLB to ensure that the newest mapping is used.
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    #[inline]
    pub fn flush(self) {
        crate::xxx::instructions::tlb::flush(self.0.start_address());
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    #[inline]
    pub fn ignore(self) {}
}

/// This type represents a change of a page table requiring a complete TLB flush
///
/// The old mapping might be still cached in the translation lookaside buffer (TLB), so it needs
/// to be flushed from the TLB before it's accessed. This type is returned from a function that
/// made the change to ensure that the TLB flush is not forgotten.
#[derive(Debug, Default)]
#[must_use = "Page Table changes must be flushed or ignored."]
pub struct MapperFlushAll(());

impl MapperFlushAll {
    /// Create a new flush promise
    ///
    /// Note that this method is intended for implementing the [`Mapper`] trait and no other uses
    /// are expected.
    #[inline]
    pub fn new() -> Self {
        MapperFlushAll(())
    }

    /// Flush all pages from the TLB to ensure that the newest mapping is used.
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    #[inline]
    pub fn flush_all(self) {
        crate::xxx::instructions::tlb::flush_all()
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    #[inline]
    pub fn ignore(self) {}
}

/// This error is returned from `map_to` and similar methods.
#[derive(Debug)]
pub enum MapToError<S: PageSize> {
    /// An additional frame was needed for the mapping process, but the frame allocator
    /// returned `None`.
    FrameAllocationFailed,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of an already mapped huge page.
    ParentEntryHugePage,
    /// The given page is already mapped to a physical frame.
    PageAlreadyMapped(PhysFrame<S>),
}

/// An error indicating that an `unmap` call failed.
#[derive(Debug)]
pub enum UnmapError {
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

/// An error indicating that an `update_flags` call failed.
#[derive(Debug)]
pub enum FlagUpdateError {
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
}

/// An error indicating that an `translate` call failed.
#[derive(Debug)]
pub enum TranslateError {
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

static _ASSERT_OBJECT_SAFE: Option<&(dyn Translate + Sync)> = None;

/// Provides methods for cleaning up unused entries.
pub trait CleanUp {
    /// Remove all empty P1-P3 tables
    ///
    /// ## Safety
    ///
    /// The caller has to guarantee that it's safe to free page table frames:
    /// All page table frames must only be used once and only in this page table
    /// (e.g. no reference counted page tables or reusing the same page tables for different virtual addresses ranges in the same page table).
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>;

    unsafe fn clean_up_addr_range<D>(
        &mut self,
        range: PageRangeInclusive,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>;
}
