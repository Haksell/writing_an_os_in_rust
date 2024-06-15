mod boot_loader_name;
mod command_line;
mod efi;
mod elf_sections;
mod end;
mod framebuffer;
mod image_load_addr;
mod memory_map;
mod module;
mod rsdp;
mod smbios;
mod tag;
mod tag_trait;
mod tag_type;
mod vbe_info;

pub use elf_sections::{ElfSection, ElfSectionFlags, ElfSectionIter, ElfSectionsTag};
use end::EndTag;
pub use memory_map::{MemoryArea, MemoryMapTag};
use tag::Tag;
use tag_trait::TagTrait;
use tag_type::{TagType, TagTypeId};

use core::mem::size_of;
use tag::TagIter;

/// Error type that describes errors while loading/parsing a multiboot2 information structure
/// from a given address.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MbiLoadError {
    IllegalAddress,
    IllegalTotalSize(u32),
    NoEndTag,
}

/// The basic header of a boot information.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootInformationHeader {
    // size is multiple of 8
    pub total_size: u32,
    _reserved: u32,
    // Followed by the boot information tags.
}

#[derive(ptr_meta::Pointee)]
#[repr(C)]
struct BootInformationInner {
    header: BootInformationHeader,
    tags: [u8],
}

impl BootInformationInner {
    /// Checks if the MBI has a valid end tag by checking the end of the mbi's
    /// bytes.
    fn has_valid_end_tag(&self) -> bool {
        let end_tag_prototype = EndTag::default();

        let self_ptr = unsafe { self.tags.as_ptr().sub(size_of::<BootInformationHeader>()) };

        let end_tag_ptr = unsafe {
            self_ptr
                .add(self.header.total_size as usize)
                .sub(size_of::<EndTag>())
        };
        let end_tag = unsafe { &*(end_tag_ptr as *const EndTag) };

        end_tag.typ == end_tag_prototype.typ && end_tag.size == end_tag_prototype.size
    }
}

/// A Multiboot 2 Boot Information (MBI) accessor.
#[repr(transparent)]
pub struct BootInformation<'a>(&'a BootInformationInner);

impl<'a> BootInformation<'a> {
    pub unsafe fn load(ptr: *const BootInformationHeader) -> Result<Self, MbiLoadError> {
        // null or not aligned
        if ptr.is_null() || ptr.align_offset(8) != 0 {
            return Err(MbiLoadError::IllegalAddress);
        }

        // mbi: reference to basic header
        let mbi = &*ptr;

        // Check if total size is not 0 and a multiple of 8.
        if mbi.total_size == 0 || mbi.total_size & 0b111 != 0 {
            return Err(MbiLoadError::IllegalTotalSize(mbi.total_size));
        }

        let slice_size = mbi.total_size as usize - size_of::<BootInformationHeader>();
        // mbi: reference to full mbi
        let mbi = ptr_meta::from_raw_parts::<BootInformationInner>(ptr.cast(), slice_size);
        let mbi = &*mbi;

        if !mbi.has_valid_end_tag() {
            return Err(MbiLoadError::NoEndTag);
        }

        Ok(Self(mbi))
    }

    /// Get the start address of the boot info.
    pub fn start_address(&self) -> usize {
        self.as_ptr() as usize
    }

    /// Get the start address of the boot info as pointer.
    pub fn as_ptr(&self) -> *const () {
        core::ptr::addr_of!(*self.0).cast()
    }

    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    /// Get the total size of the boot info struct.
    pub fn total_size(&self) -> usize {
        self.0.header.total_size as usize
    }

    pub fn elf_sections(&self) -> Option<ElfSectionIter> {
        let tag = self.get_tag::<ElfSectionsTag>();
        tag.map(|t| {
            assert!((t.entry_size * t.shndx) <= t.size);
            t.sections()
        })
    }

    /// Search for the Memory map tag.
    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag::<MemoryMapTag>()
    }

    pub fn get_tag<TagT: TagTrait + ?Sized + 'a>(&'a self) -> Option<&'a TagT> {
        self.tags()
            .find(|tag| tag.typ == TagT::ID)
            .map(|tag| tag.cast_tag::<TagT>())
    }

    /// Returns an iterator over all tags.
    fn tags(&self) -> TagIter {
        TagIter::new(&self.0.tags)
    }
}
