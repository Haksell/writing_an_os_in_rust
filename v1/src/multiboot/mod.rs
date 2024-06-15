mod elf_sections;
mod memory_map;
mod tag;

pub use elf_sections::{ElfSection, ElfSectionFlags};
pub use memory_map::MemoryArea;

use self::elf_sections::{ElfSectionIter, ElfSectionsTag};
use self::memory_map::MemoryMapTag;
use self::tag::{Tag, TagIter, TagTrait, TagType};

pub struct BootInformation {
    pub start_address: usize,
    pub end_address: usize,
    tags: &'static [u8],
}

impl BootInformation {
    pub unsafe fn load(multiboot_address: usize) -> Self {
        let total_size = *(multiboot_address as *const u32) as usize;
        Self {
            start_address: multiboot_address,
            end_address: multiboot_address + total_size,
            tags: core::slice::from_raw_parts((multiboot_address + 8) as *const u8, total_size - 8),
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
        TagIter::new(self.tags)
            .find(|tag| tag.typ == TagT::ID.into())
            .map(|tag| tag.cast_tag::<TagT>())
    }
}
