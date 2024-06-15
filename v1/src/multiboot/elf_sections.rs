use super::{Tag, TagTrait, TagType};
use bitflags::bitflags;
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<u32>() + 4 * size_of::<u32>();

#[derive(ptr_meta::Pointee)]
#[repr(C)]
pub struct ElfSectionsTag {
    typ: u32,
    pub size: u32,
    number_of_sections: u32,
    pub entry_size: u32,
    shndx: u32,
    sections: [u8],
}

impl ElfSectionsTag {
    pub fn sections(&self) -> ElfSectionIter {
        ElfSectionIter {
            current_section: &self.sections[0],
            remaining_sections: self.number_of_sections,
        }
    }
}

impl TagTrait for ElfSectionsTag {
    const ID: TagType = TagType::ElfSections;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

pub struct ElfSectionIter {
    current_section: *const u8,
    remaining_sections: u32,
}

impl Iterator for ElfSectionIter {
    type Item = ElfSection;

    fn next(&mut self) -> Option<ElfSection> {
        while self.remaining_sections != 0 {
            let section = ElfSection {
                inner: self.current_section,
            };
            self.current_section = unsafe { self.current_section.offset(64) };
            self.remaining_sections -= 1;
            if section.is_used() {
                return Some(section);
            }
        }
        None
    }
}

pub struct ElfSection {
    inner: *const u8,
}

#[repr(C, packed)]
struct ElfSectionInner64 {
    name_index: u32,
    typ: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entry_size: u64,
}

impl ElfSection {
    pub fn start_address(&self) -> u64 {
        self.get().addr
    }

    pub fn end_address(&self) -> u64 {
        self.get().addr + self.get().size
    }

    pub fn size(&self) -> u64 {
        self.get().size
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.get().flags)
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ElfSectionFlags::ALLOCATED)
    }

    fn is_used(&self) -> bool {
        self.get().typ != 0
    }

    fn get(&self) -> &ElfSectionInner64 {
        unsafe { &*(self.inner as *const ElfSectionInner64) }
    }
}

bitflags! {
    pub struct ElfSectionFlags: u64 {
        const WRITABLE = 0x1;
        const ALLOCATED = 0x2;
        const EXECUTABLE = 0x4;
    }
}
