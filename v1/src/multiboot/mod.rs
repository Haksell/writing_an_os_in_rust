mod elf_sections;
mod memory_map;
mod tag;

pub use self::{
    elf_sections::{ElfSection, ElfSectionFlags},
    memory_map::MemoryArea,
};
use self::{
    elf_sections::{ElfSectionIter, ElfSectionsTag},
    memory_map::MemoryMapTag,
    tag::{Tag, TagIter, TagTrait, TagType},
};

pub struct MultiBoot {
    pub start_address: usize,
    pub end_address: usize,
    tags: &'static [u8],
}

impl MultiBoot {
    pub unsafe fn load(multiboot_address: usize) -> Self {
        let total_size = *(multiboot_address as *const u32) as usize;
        Self {
            start_address: multiboot_address,
            end_address: multiboot_address + total_size,
            tags: core::slice::from_raw_parts((multiboot_address + 8) as *const u8, total_size - 8),
        }
    }

    pub fn elf_sections(&self) -> ElfSectionIter {
        self.get_tag::<ElfSectionsTag>().sections()
    }

    pub fn memory_areas(&self) -> &[MemoryArea] {
        &self.get_tag::<MemoryMapTag>().areas
    }

    fn get_tag<T: TagTrait + ?Sized>(&self) -> &T {
        TagIter::new(self.tags)
            .find(|tag| tag.typ == T::ID.into())
            .unwrap()
            .cast_tag::<T>()
    }
}
