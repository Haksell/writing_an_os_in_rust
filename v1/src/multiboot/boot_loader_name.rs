use super::{Tag, TagTrait, TagType, TagTypeId};
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<TagTypeId>() + size_of::<u32>();

/// The bootloader name tag.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootLoaderNameTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    name: [u8],
}

impl TagTrait for BootLoaderNameTag {
    const ID: TagType = TagType::BootLoaderName;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}
