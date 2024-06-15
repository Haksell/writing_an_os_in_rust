use super::{Tag, TagTrait, TagType, TagTypeId};
use core::mem;

const METADATA_SIZE: usize = mem::size_of::<TagTypeId>() + 3 * mem::size_of::<u32>();
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct MemoryArea {
    base_addr: u64,
    length: u64,
    typ: MemoryAreaTypeId,
    _reserved: u32,
}

impl MemoryArea {
    pub fn start_address(&self) -> u64 {
        self.base_addr
    }
    pub fn size(&self) -> u64 {
        self.length
    }
}
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MemoryAreaType {
    Available,
    Reserved,
    AcpiAvailable,
    ReservedHibernate,
    Defective,
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
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
struct EFIMemoryMapTag {
    typ: TagTypeId,
    size: u32,
    desc_size: u32,
    desc_version: u32,
    memory_map: [u8],
}

impl TagTrait for EFIMemoryMapTag {
    const ID: TagType = TagType::EfiMmap;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= EFI_METADATA_SIZE);
        base_tag.size as usize - EFI_METADATA_SIZE
    }
}
