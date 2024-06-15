use super::{Tag, TagType};
use ptr_meta::Pointee;

pub trait TagTrait: Pointee {
    const ID: TagType;

    fn dst_size(base_tag: &Tag) -> Self::Metadata;

    unsafe fn from_base_tag(tag: &Tag) -> &Self {
        let ptr = core::ptr::addr_of!(*tag);
        let ptr = ptr_meta::from_raw_parts(ptr.cast(), Self::dst_size(tag));
        &*ptr
    }
}
