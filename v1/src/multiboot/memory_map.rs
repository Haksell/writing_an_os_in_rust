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
    areas: [MemoryArea],
}

impl MemoryMapTag {
    pub fn memory_areas(&self) -> &[MemoryArea] {
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
    typ: u32,
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
