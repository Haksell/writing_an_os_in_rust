use super::TagType;
use core::marker::PhantomData;
use ptr_meta::Pointee;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tag {
    pub typ: u32,
    pub size: u32,
    // followed by additional, tag specific fields
}

impl Tag {
    fn typ(&self) -> TagType {
        self.typ.into()
    }

    pub fn cast_tag<'a, T: TagTrait + ?Sized + 'a>(&'a self) -> &'a T {
        assert_eq!(self.typ, T::ID);
        // Safety: At this point, we trust that "self.size" and the size hint
        // for DST tags are sane.
        unsafe { TagTrait::from_base_tag(self) }
    }
}

#[derive(Clone)]
pub struct TagIter<'a> {
    current: *const Tag,
    _mem: PhantomData<&'a ()>,
}

impl<'a> TagIter<'a> {
    pub fn new(mem: &'a [u8]) -> Self {
        assert_eq!(mem.as_ptr().align_offset(8), 0);
        TagIter {
            current: mem.as_ptr().cast(),
            _mem: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        let tag = unsafe { &*self.current };
        match tag.typ() {
            TagType::End => None,
            _ => {
                let ptr_offset = (tag.size as usize + 7) & !7;
                self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };
                Some(tag)
            }
        }
    }
}

pub trait TagTrait: Pointee {
    const ID: TagType;

    fn dst_size(base_tag: &Tag) -> Self::Metadata;

    unsafe fn from_base_tag(tag: &Tag) -> &Self {
        let ptr = core::ptr::addr_of!(*tag);
        let ptr = ptr_meta::from_raw_parts(ptr.cast(), Self::dst_size(tag));
        &*ptr
    }
}
