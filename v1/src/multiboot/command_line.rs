//! Module for [`CommandLineTag`].

use super::{Tag, TagTrait, TagType, TagTypeId};

use super::tag::StringError;
use core::fmt::{Debug, Formatter};
use core::mem;
use core::str;

pub(crate) const METADATA_SIZE: usize = mem::size_of::<TagTypeId>() + mem::size_of::<u32>();

/// This tag contains the command line string.
///
/// The string is a normal C-style UTF-8 zero-terminated string that can be
/// obtained via the `command_line` method.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct CommandLineTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    cmdline: [u8],
}

impl CommandLineTag {
    pub fn cmdline(&self) -> Result<&str, StringError> {
        Tag::parse_slice_as_string(&self.cmdline)
    }
}

impl Debug for CommandLineTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommandLineTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("cmdline", &self.cmdline())
            .finish()
    }
}

impl TagTrait for CommandLineTag {
    const ID: TagType = TagType::Cmdline;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}