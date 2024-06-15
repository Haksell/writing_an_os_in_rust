use super::{Tag, TagTrait, TagType};
use core::mem;

const METADATA_SIZE: usize = mem::size_of::<u32>() + 3 * mem::size_of::<u32>();

#[derive(ptr_meta::Pointee, PartialEq, Eq)]
#[repr(C)]
pub struct MemoryMapTag {
    typ: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    pub areas: [MemoryArea],
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
    pub start_address: u64,
    pub size: u64,
    typ: u32,
    _reserved: u32,
}
