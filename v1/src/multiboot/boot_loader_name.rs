//! Module for [`BootLoaderNameTag`].

use super::tag::StringError;
use super::{Tag, TagTrait, TagType, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<TagTypeId>() + size_of::<u32>();

/// The bootloader name tag.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootLoaderNameTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    name: [u8],
}

impl BootLoaderNameTag {
    /// Reads the name of the bootloader that is booting the kernel as Rust
    /// string slice without the null-byte.
    ///
    /// For example, this returns `"GRUB 2.02~beta3-5"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// if let Some(tag) = boot_info.boot_loader_name_tag() {
    ///     assert_eq!(Ok("GRUB 2.02~beta3-5"), tag.name());
    /// }
    /// ```
    pub fn name(&self) -> Result<&str, StringError> {
        Tag::parse_slice_as_string(&self.name)
    }
}

impl Debug for BootLoaderNameTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("name", &self.name())
            .finish()
    }
}

impl TagTrait for BootLoaderNameTag {
    const ID: TagType = TagType::BootLoaderName;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}
