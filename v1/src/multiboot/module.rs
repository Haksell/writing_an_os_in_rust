use super::{Tag, TagIter, TagTrait, TagType, TagTypeId};
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<TagTypeId>() + 3 * size_of::<u32>();

/// The module tag can occur multiple times and specifies passed boot modules
/// (blobs in memory). The tag itself doesn't include the blog, but references
/// its location.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ModuleTag {
    typ: TagTypeId,
    size: u32,
    mod_start: u32,
    mod_end: u32,
    /// Null-terminated UTF-8 string
    cmdline: [u8],
}

impl TagTrait for ModuleTag {
    const ID: TagType = TagType::Module;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

/// An iterator over all module tags.
#[derive(Clone)]
pub struct ModuleIter<'a> {
    iter: TagIter<'a>,
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = &'a ModuleTag;

    fn next(&mut self) -> Option<&'a ModuleTag> {
        self.iter
            .find(|tag| tag.typ == TagType::Module)
            .map(|tag| tag.cast_tag())
    }
}
