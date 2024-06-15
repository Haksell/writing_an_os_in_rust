use super::{Tag, TagTrait, TagType, TagTypeId};

#[repr(C)]
pub struct EndTag {
    pub typ: TagTypeId,
    pub size: u32,
}

impl Default for EndTag {
    fn default() -> Self {
        Self {
            typ: TagType::End.into(),
            size: 8,
        }
    }
}

impl TagTrait for EndTag {
    const ID: TagType = TagType::End;

    fn dst_size(_base_tag: &Tag) {}
}
