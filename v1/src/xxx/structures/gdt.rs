//! Types for the Global Descriptor Table and segment selectors.

pub use crate::xxx::registers::segmentation::SegmentSelector;
use crate::xxx::structures::tss::TaskStateSegment;
use crate::xxx::PrivilegeLevel;
use bit_field::BitField;
use bitflags::bitflags;
use core::fmt;

use core::sync::atomic::{AtomicU64 as EntryValue, Ordering};

/// 8-byte entry in a descriptor table.
///
/// A [`GlobalDescriptorTable`] (or LDT) is an array of these entries, and
/// [`SegmentSelector`]s index into this array. Each [`Descriptor`] in the table
/// uses either 1 Entry (if it is a [`UserSegment`](Descriptor::UserSegment)) or
/// 2 Entries (if it is a [`SystemSegment`](Descriptor::SystemSegment)). This
/// type exists to give users access to the raw entry bits in a GDT.
#[repr(transparent)]
pub struct Entry(EntryValue);

impl Entry {
    const fn new(raw: u64) -> Self {
        #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
        let raw = EntryValue::new(raw);
        Self(raw)
    }

    pub fn raw(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self::new(self.raw())
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for Entry {}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display inner value as hex
        write!(f, "Entry({:#018x})", self.raw())
    }
}

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable<const MAX: usize = 8> {
    table: [Entry; MAX],
    len: usize,
}

impl GlobalDescriptorTable {
    /// Creates an empty GDT with the default length of 8.
    pub const fn new() -> Self {
        Self::empty()
    }
}

impl Default for GlobalDescriptorTable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX: usize> GlobalDescriptorTable<MAX> {
    /// Creates an empty GDT which can hold `MAX` number of [`Entry`]s.
    #[inline]
    pub const fn empty() -> Self {
        // TODO: Replace with compiler error when feature(generic_const_exprs) is stable.
        assert!(MAX > 0, "A GDT cannot have 0 entries");
        assert!(MAX <= (1 << 13), "A GDT can only have at most 2^13 entries");

        // TODO: Replace with inline_const when it's stable.
        #[allow(clippy::declare_interior_mutable_const)]
        const NULL: Entry = Entry::new(0);
        Self {
            table: [NULL; MAX],
            len: 1,
        }
    }
}

/// A 64-bit mode segment descriptor.
///
/// Segmentation is no longer supported in 64-bit mode, so most of the descriptor
/// contents are ignored.
#[derive(Debug, Clone, Copy)]
pub enum Descriptor {
    /// Descriptor for a code or data segment.
    ///
    /// Since segmentation is no longer supported in 64-bit mode, almost all of
    /// code and data descriptors is ignored. Only some flags are still used.
    UserSegment(u64),
    /// A system segment descriptor such as a LDT or TSS descriptor.
    SystemSegment(u64, u64),
}

bitflags! {
    /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct DescriptorFlags: u64 {
        /// Set by the processor if this segment has been accessed. Only cleared by software.
        /// _Setting_ this bit in software prevents GDT writes on first use.
        const ACCESSED          = 1 << 40;
        /// For 32-bit data segments, sets the segment as writable. For 32-bit code segments,
        /// sets the segment as _readable_. In 64-bit mode, ignored for all segments.
        const WRITABLE          = 1 << 41;
        /// For code segments, sets the segment as “conforming”, influencing the
        /// privilege checks that occur on control transfers. For 32-bit data segments,
        /// sets the segment as "expand down". In 64-bit mode, ignored for data segments.
        const CONFORMING        = 1 << 42;
        /// This flag must be set for code segments and unset for data segments.
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// These two bits encode the Descriptor Privilege Level (DPL) for this descriptor.
        /// If both bits are set, the DPL is Ring 3, if both are unset, the DPL is Ring 0.
        const DPL_RING_3        = 3 << 45;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Available for use by the Operating System
        const AVAILABLE         = 1 << 52;
        /// Must be set for 64-bit code segments, unset otherwise.
        const LONG_MODE         = 1 << 53;
        /// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][Self::LONG_MODE] is set,
        /// this must be unset. In 64-bit mode, ignored for data segments.
        const DEFAULT_SIZE      = 1 << 54;
        /// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all segments.
        const GRANULARITY       = 1 << 55;

        /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
        const LIMIT_0_15        = 0xFFFF;
        /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
        const LIMIT_16_19       = 0xF << 48;
        /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_0_23         = 0xFF_FFFF << 16;
        /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_24_31        = 0xFF << 56;
    }
}

/// The following constants define default values for common GDT entries. They
/// are all "flat" segments, meaning they can access the entire address space.
/// These values all set [`WRITABLE`][DescriptorFlags::WRITABLE] and
/// [`ACCESSED`][DescriptorFlags::ACCESSED]. They also match the values loaded
/// by the `syscall`/`sysret` and `sysenter`/`sysexit` instructions.
///
/// In short, these values disable segmentation, permission checks, and access
/// tracking at the GDT level. Kernels using these values should use paging to
/// implement this functionality.
impl DescriptorFlags {
    // Flags that we set for all our default segments
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );
    /// A kernel data segment (64-bit or flat 32-bit)
    pub const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::COMMON.bits() | Self::DEFAULT_SIZE.bits());
    /// A flat 32-bit kernel code segment
    pub const KERNEL_CODE32: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::DEFAULT_SIZE.bits(),
    );
    /// A 64-bit kernel code segment
    pub const KERNEL_CODE64: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::LONG_MODE.bits(),
    );
    /// A user data segment (64-bit or flat 32-bit)
    pub const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());
    /// A flat 32-bit user code segment
    pub const USER_CODE32: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE32.bits() | Self::DPL_RING_3.bits());
    /// A 64-bit user code segment
    pub const USER_CODE64: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE64.bits() | Self::DPL_RING_3.bits());
}

impl Descriptor {
    /// Returns the Descriptor Privilege Level (DPL). When using this descriptor
    /// via a [`SegmentSelector`], the RPL and Current Privilege Level (CPL)
    /// must less than or equal to the DPL, except for stack segments where the
    /// RPL, CPL, and DPL must all be equal.
    #[inline]
    pub const fn dpl(self) -> PrivilegeLevel {
        let value_low = match self {
            Descriptor::UserSegment(v) => v,
            Descriptor::SystemSegment(v, _) => v,
        };
        let dpl = (value_low & DescriptorFlags::DPL_RING_3.bits()) >> 45;
        PrivilegeLevel::from_u16(dpl as u16)
    }

    /// Creates a segment descriptor for a 64-bit kernel code segment. Suitable
    /// for use with `syscall` or 64-bit `sysenter`.
    #[inline]
    pub const fn kernel_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE64.bits())
    }

    /// Creates a segment descriptor for a kernel data segment (32-bit or
    /// 64-bit). Suitable for use with `syscall` or `sysenter`.
    #[inline]
    pub const fn kernel_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits())
    }

    /// Creates a segment descriptor for a ring 3 data segment (32-bit or
    /// 64-bit). Suitable for use with `sysret` or `sysexit`.
    #[inline]
    pub const fn user_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::USER_DATA.bits())
    }

    /// Creates a segment descriptor for a 64-bit ring 3 code segment. Suitable
    /// for use with `sysret` or `sysexit`.
    #[inline]
    pub const fn user_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::USER_CODE64.bits())
    }

    /// Creates a TSS system descriptor for the given TSS.
    ///
    /// While it is possible to create multiple Descriptors that point to the
    /// same TSS, this generally isn't recommended, as the TSS usually contains
    /// per-CPU information such as the RSP and IST pointers. Instead, there
    /// should be exactly one TSS and one corresponding TSS Descriptor per CPU.
    /// Then, each of these descriptors should be placed in a GDT (which can
    /// either be global or per-CPU).
    #[inline]
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        // SAFETY: The pointer is derived from a &'static reference, which ensures its validity.
        unsafe { Self::tss_segment_unchecked(tss) }
    }

    /// Similar to [`Descriptor::tss_segment`], but unsafe since it does not enforce a lifetime
    /// constraint on the provided TSS.
    ///
    /// # Safety
    /// The caller must ensure that the passed pointer is valid for as long as the descriptor is
    /// being used.
    #[inline]
    pub unsafe fn tss_segment_unchecked(tss: *const TaskStateSegment) -> Descriptor {
        use self::DescriptorFlags as Flags;
        use core::mem::size_of;

        let ptr = tss as u64;

        let mut low = Flags::PRESENT.bits();
        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (the `-1` in needed since the bound is inclusive)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}
