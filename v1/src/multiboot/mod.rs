mod elf_sections;
mod memory_map;
mod tag;
mod tag_trait;
mod tag_type;

pub use elf_sections::{ElfSection, ElfSectionFlags};
pub use memory_map::MemoryArea;

use self::elf_sections::{ElfSectionIter, ElfSectionsTag};
use self::memory_map::MemoryMapTag;
use self::tag::{Tag, TagIter};
use self::tag_trait::TagTrait;
use self::tag_type::{TagType, TagTypeId};
use core::mem::size_of;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
struct BootInformationHeader {
    // size is multiple of 8
    total_size: u32,
    _reserved: u32,
    // Followed by the boot information tags.
}

// TODO: remove this crap
#[derive(ptr_meta::Pointee)]
#[repr(C)]
struct BootInformationInner {
    header: BootInformationHeader,
    tags: [u8],
}

#[repr(transparent)]
pub struct BootInformation<'a>(&'a BootInformationInner);

impl<'a> BootInformation<'a> {
    pub unsafe fn load(ptr: usize) -> Self {
        let ptr = ptr as *const BootInformationHeader;
        let mbi = &*ptr;
        let slice_size = mbi.total_size as usize - size_of::<BootInformationHeader>();
        let mbi = ptr_meta::from_raw_parts::<BootInformationInner>(ptr.cast(), slice_size);
        Self(&*mbi)
    }

    pub fn start_address(&self) -> usize {
        core::ptr::addr_of!(*self.0).cast() as *const () as usize
    }

    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    fn total_size(&self) -> usize {
        self.0.header.total_size as usize
    }

    pub fn elf_sections(&self) -> Option<ElfSectionIter> {
        self.get_tag::<ElfSectionsTag>().map(|t| {
            assert!((t.entry_size * t.shndx) <= t.size);
            t.sections()
        })
    }

    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag::<MemoryMapTag>()
    }

    fn get_tag<TagT: TagTrait + ?Sized + 'a>(&'a self) -> Option<&'a TagT> {
        self.tags()
            .find(|tag| tag.typ == TagT::ID)
            .map(|tag| tag.cast_tag::<TagT>())
    }

    fn tags(&self) -> TagIter {
        TagIter::new(&self.0.tags)
    }
}
