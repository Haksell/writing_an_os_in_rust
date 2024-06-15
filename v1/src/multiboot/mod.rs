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

pub struct BootInformation {
    pub start_address: usize,
    pub end_address: usize,
    tags: &'static [u8],
}

impl BootInformation {
    pub unsafe fn load(multiboot_address: usize) -> Self {
        let total_size = *(multiboot_address as *const u32);
        let tags_ptr = (multiboot_address + size_of::<u32>() * 2) as *const u8;
        let slice_size = total_size as usize - size_of::<u32>() * 2;
        Self {
            start_address: multiboot_address,
            end_address: multiboot_address + total_size as usize,
            tags: core::slice::from_raw_parts(tags_ptr, slice_size),
        }
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

    fn get_tag<TagT: TagTrait + ?Sized>(&self) -> Option<&TagT> {
        self.tags()
            .find(|tag| tag.typ == TagT::ID)
            .map(|tag| tag.cast_tag::<TagT>())
    }

    fn tags(&self) -> TagIter {
        TagIter::new(self.tags)
    }
}
