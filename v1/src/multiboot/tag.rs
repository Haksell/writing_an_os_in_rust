use super::{TagTrait, TagType, TagTypeId};
use core::marker::PhantomData;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tag {
    pub typ: TagTypeId, // u32
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
    end_ptr_exclusive: *const u8,
    _mem: PhantomData<&'a ()>,
}

impl<'a> TagIter<'a> {
    pub fn new(mem: &'a [u8]) -> Self {
        assert_eq!(mem.as_ptr().align_offset(8), 0);
        TagIter {
            current: mem.as_ptr().cast(),
            end_ptr_exclusive: unsafe { mem.as_ptr().add(mem.len()) },
            _mem: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        // This never failed so far. But better be safe.
        assert!(self.current.cast::<u8>() < self.end_ptr_exclusive);

        let tag = unsafe { &*self.current };
        match tag.typ() {
            TagType::End => None, // end tag
            _ => {
                // We return the tag and update self.current already to the next
                // tag.

                // next pointer (rounded up to 8-byte alignment)
                let ptr_offset = (tag.size as usize + 7) & !7;

                // go to next tag
                self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };

                Some(tag)
            }
        }
    }
}
