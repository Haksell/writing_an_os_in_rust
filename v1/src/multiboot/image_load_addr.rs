//! Module for [`ImageLoadPhysAddrTag`].

use super::{Tag, TagTrait, TagType, TagTypeId};

/// The physical load address tag. Typically, this is only available if the
/// binary was relocated, for example if the relocatable header tag was
/// specified.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ImageLoadPhysAddrTag {
    typ: TagTypeId,
    size: u32,
    load_base_addr: u32,
}

impl TagTrait for ImageLoadPhysAddrTag {
    const ID: TagType = TagType::LoadBaseAddr;

    fn dst_size(_base_tag: &Tag) {}
}
