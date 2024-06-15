//! Module for [`SmbiosTag`].

use super::{Tag, TagTrait, TagType, TagTypeId};
use core::fmt::Debug;

const METADATA_SIZE: usize = core::mem::size_of::<TagTypeId>()
    + core::mem::size_of::<u32>()
    + core::mem::size_of::<u8>() * 8;

/// This tag contains a copy of SMBIOS tables as well as their version.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct SmbiosTag {
    typ: TagTypeId,
    size: u32,
    pub major: u8,
    pub minor: u8,
    _reserved: [u8; 6],
    pub tables: [u8],
}

impl TagTrait for SmbiosTag {
    const ID: TagType = TagType::Smbios;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

impl Debug for SmbiosTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("major", &{ self.major })
            .field("minor", &{ self.minor })
            .finish()
    }
}
