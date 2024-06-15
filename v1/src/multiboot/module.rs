//! Module for [`ModuleTag`].

use super::tag::StringError;
use super::{Tag, TagIter, TagTrait, TagType, TagTypeId};
use core::fmt::{Debug, Formatter};
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

impl ModuleTag {
    /// Reads the command line of the boot module as Rust string slice without
    /// the null-byte.
    /// This is an null-terminated UTF-8 string. If this returns `Err` then perhaps the memory
    /// is invalid or the bootloader doesn't follow the spec.
    ///
    /// For example, this returns `"--test cmdline-option"`.if the GRUB config
    /// contains  `"module2 /some_boot_module --test cmdline-option"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    pub fn cmdline(&self) -> Result<&str, StringError> {
        Tag::parse_slice_as_string(&self.cmdline)
    }

    /// The size of the module/the BLOB in memory.
    pub fn module_size(&self) -> u32 {
        self.mod_end - self.mod_start
    }
}

impl TagTrait for ModuleTag {
    const ID: TagType = TagType::Module;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

impl Debug for ModuleTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleTag")
            .field("type", &{ self.typ })
            .field("size", &{ self.size })
            // Trick to print as hex.
            .field("mod_start", &self.mod_start)
            .field("mod_end", &self.mod_end)
            .field("mod_size", &self.module_size())
            .field("cmdline", &self.cmdline())
            .finish()
    }
}

pub fn module_iter(iter: TagIter) -> ModuleIter {
    ModuleIter { iter }
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

impl<'a> Debug for ModuleIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        self.clone().for_each(|tag| {
            list.entry(&tag);
        });
        list.finish()
    }
}
