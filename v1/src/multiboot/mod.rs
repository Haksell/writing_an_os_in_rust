mod elf_sections;
mod memory_map;
mod tag;
mod tag_type;

pub use elf_sections::{ElfSection, ElfSectionFlags};
pub use memory_map::MemoryArea;

use self::elf_sections::{ElfSectionIter, ElfSectionsTag};
use self::memory_map::MemoryMapTag;
use self::tag::{Tag, TagIter, TagTrait};
use self::tag_type::TagType;
use core::mem::size_of;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
struct BootInformationHeader {
    total_size: u32,
    _reserved: u32,
}

pub struct BootInformation<'a> {
    header: &'a BootInformationHeader,
    tags: &'a [u8],
}

impl<'a> BootInformation<'a> {
    pub unsafe fn load(ptr: usize) -> Self {
        let header_ptr = ptr as *const BootInformationHeader;
        let header = &*header_ptr;
        let slice_size = header.total_size as usize - size_of::<BootInformationHeader>();
        let tags_ptr = header_ptr.add(1) as *const u8;
        let tags = core::slice::from_raw_parts(tags_ptr, slice_size);
        Self { header, tags }
    }

    pub fn start_address(&self) -> usize {
        self.header as *const _ as usize
    }

    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    fn total_size(&self) -> usize {
        self.header.total_size as usize
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
        TagIter::new(self.tags)
    }
}
