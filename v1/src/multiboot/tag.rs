//! Module for the base tag definitions and helper types.
//!
//! The relevant exports of this module is [`Tag`].

use super::{TagTrait, TagType, TagTypeId};
use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
use core::str::Utf8Error;

/// Error type describing failures when parsing the string from a tag.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringError {
    /// There is no terminating NUL character, although the specification
    /// requires one.
    MissingNul(core::ffi::FromBytesUntilNulError),
    /// The sequence until the first NUL character is not valid UTF-8.
    Utf8(Utf8Error),
}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tag {
    pub typ: TagTypeId, // u32
    pub size: u32,
    // followed by additional, tag specific fields
}

impl Tag {
    /// Returns the underlying type of the tag.
    pub fn typ(&self) -> TagType {
        self.typ.into()
    }

    /// Casts the base tag to the specific tag type.
    pub fn cast_tag<'a, T: TagTrait + ?Sized + 'a>(&'a self) -> &'a T {
        assert_eq!(self.typ, T::ID);
        // Safety: At this point, we trust that "self.size" and the size hint
        // for DST tags are sane.
        unsafe { TagTrait::from_base_tag(self) }
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let tag_type = TagType::from(self.typ);

        let mut debug = f.debug_struct("Tag");
        debug.field("typ", &tag_type);

        if !matches!(tag_type, TagType::Custom(_)) {
            debug.field("typ (numeric)", &(u32::from(self.typ)));
        }

        debug.field("size", &(self.size));

        debug.finish()
    }
}

/// Iterates the MBI's tags from the first tag to the end tag.
#[derive(Clone, Debug)]
pub struct TagIter<'a> {
    /// Pointer to the next tag. Updated in each iteration.
    pub current: *const Tag,
    /// The pointer right after the MBI. Used for additional bounds checking.
    end_ptr_exclusive: *const u8,
    /// Lifetime capture of the MBI's memory.
    _mem: PhantomData<&'a ()>,
}

impl<'a> TagIter<'a> {
    /// Creates a new iterator
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
