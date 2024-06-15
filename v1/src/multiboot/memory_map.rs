use super::{Tag, TagTrait, TagType, TagTypeId};
use core::mem;

const METADATA_SIZE: usize = mem::size_of::<TagTypeId>() + 3 * mem::size_of::<u32>();

/// This tag provides an initial host memory map (legacy boot, not UEFI).
///
/// The map provided is guaranteed to list all standard RAM that should be
/// available for normal use. This type however includes the regions occupied
/// by kernel, mbi, segments and modules. Kernel must take care not to
/// overwrite these regions.
///
/// This tag may not be provided by some boot loaders on EFI platforms if EFI
/// boot services are enabled and available for the loaded image (The EFI boot
/// services tag may exist in the Multiboot2 boot information structure).
#[derive(ptr_meta::Pointee, PartialEq, Eq)]
#[repr(C)]
pub struct MemoryMapTag {
    typ: TagTypeId,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    areas: [MemoryArea],
}

impl MemoryMapTag {
    pub fn memory_areas(&self) -> &[MemoryArea] {
        // If this ever fails, we need to model this differently in this crate.
        assert_eq!(self.entry_size as usize, mem::size_of::<MemoryArea>());
        &self.areas
    }
}

impl TagTrait for MemoryMapTag {
    const ID: TagType = TagType::Mmap;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        let size = base_tag.size as usize - METADATA_SIZE;
        assert_eq!(size % mem::size_of::<MemoryArea>(), 0);
        size / mem::size_of::<MemoryArea>()
    }
}

/// A descriptor for an available or taken area of physical memory.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct MemoryArea {
    base_addr: u64,
    length: u64,
    typ: MemoryAreaTypeId,
    _reserved: u32,
}

impl MemoryArea {
    /// The start address of the memory region.
    pub fn start_address(&self) -> u64 {
        self.base_addr
    }

    /// The size, in bytes, of the memory region.
    pub fn size(&self) -> u64 {
        self.length
    }
}

/// ABI-friendly version of [`MemoryAreaType`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
struct MemoryAreaTypeId(u32);

impl From<u32> for MemoryAreaTypeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<MemoryAreaTypeId> for u32 {
    fn from(value: MemoryAreaTypeId) -> Self {
        value.0
    }
}

/// Abstraction over defined memory types for the memory map as well as custom
/// ones. Types 1 to 5 are defined in the Multiboot2 spec and correspond to the
/// entry types of e820 memory maps.
///
/// This is not binary compatible with the Multiboot2 spec. Please use
/// [`MemoryAreaTypeId`] instead.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MemoryAreaType {
    /// Available memory free to be used by the OS.
    Available, /* 1 */

    /// A reserved area that must not be used.
    Reserved, /* 2, */

    /// Usable memory holding ACPI information.
    AcpiAvailable, /* 3, */

    /// Reserved memory which needs to be preserved on hibernation.
    /// Also called NVS in spec, which stands for "Non-Volatile Sleep/Storage",
    /// which is part of ACPI specification.
    ReservedHibernate, /* 4, */

    /// Memory which is occupied by defective RAM modules.
    Defective, /* = 5, */

    /// Custom memory map type.
    Custom(u32),
}

impl From<MemoryAreaTypeId> for MemoryAreaType {
    fn from(value: MemoryAreaTypeId) -> Self {
        match value.0 {
            1 => Self::Available,
            2 => Self::Reserved,
            3 => Self::AcpiAvailable,
            4 => Self::ReservedHibernate,
            5 => Self::Defective,
            val => Self::Custom(val),
        }
    }
}

impl From<MemoryAreaType> for MemoryAreaTypeId {
    fn from(value: MemoryAreaType) -> Self {
        let integer = match value {
            MemoryAreaType::Available => 1,
            MemoryAreaType::Reserved => 2,
            MemoryAreaType::AcpiAvailable => 3,
            MemoryAreaType::ReservedHibernate => 4,
            MemoryAreaType::Defective => 5,
            MemoryAreaType::Custom(val) => val,
        };
        integer.into()
    }
}

impl PartialEq<MemoryAreaType> for MemoryAreaTypeId {
    fn eq(&self, other: &MemoryAreaType) -> bool {
        let val: MemoryAreaTypeId = (*other).into();
        let val: u32 = val.0;
        self.0.eq(&val)
    }
}

impl PartialEq<MemoryAreaTypeId> for MemoryAreaType {
    fn eq(&self, other: &MemoryAreaTypeId) -> bool {
        let val: MemoryAreaTypeId = (*self).into();
        let val: u32 = val.0;
        other.0.eq(&val)
    }
}

const EFI_METADATA_SIZE: usize = mem::size_of::<TagTypeId>() + 3 * mem::size_of::<u32>();

/// EFI memory map tag. The embedded [`EFIMemoryDesc`]s follows the EFI
/// specification.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
struct EFIMemoryMapTag {
    typ: TagTypeId,
    size: u32,
    /// Most likely a little more than the size of a [`EFIMemoryDesc`].
    /// This is always the reference, and `size_of` never.
    /// See <https://github.com/tianocore/edk2/blob/7142e648416ff5d3eac6c6d607874805f5de0ca8/MdeModulePkg/Core/PiSmmCore/Page.c#L1059>.
    desc_size: u32,
    /// Version of the tag. The spec leaves it open to extend the memory
    /// descriptor in the future. However, this never happened so far.
    /// At the moment, only version "1" is supported.
    desc_version: u32,
    /// Contains the UEFI memory map.
    ///
    /// To follow the UEFI spec and to allow extendability for future UEFI
    /// revisions, the length is a multiple of `desc_size` and not a multiple
    /// of `size_of::<EfiMemoryDescriptor>()`.
    ///
    /// This tag is properly `align_of::<EFIMemoryDesc>` aligned, if the tag
    /// itself is also 8 byte aligned, which every sane MBI guarantees.
    memory_map: [u8],
}

impl TagTrait for EFIMemoryMapTag {
    const ID: TagType = TagType::EfiMmap;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= EFI_METADATA_SIZE);
        base_tag.size as usize - EFI_METADATA_SIZE
    }
}
