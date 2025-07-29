use {
    super::{Tag, TagTrait, TagType},
    bitflags::bitflags,
    core::mem::size_of,
};

const METADATA_SIZE: usize = 5 * size_of::<u32>();

#[repr(C)]
pub struct ElfSectionsTag {
    typ: u32,
    size: u32,
    number_of_sections: u32,
    entry_size: u32,
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
            let section = unsafe { *(self.current_section as *const ElfSection) };
            self.current_section = unsafe { self.current_section.offset(64) };
            self.remaining_sections -= 1;
            if section.is_used() {
                return Some(section);
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ElfSection {
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
        self.addr
    }

    pub fn end_address(&self) -> u64 {
        self.addr + self.size
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.flags)
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ElfSectionFlags::ALLOCATED)
    }

    fn is_used(&self) -> bool {
        self.typ != 0
    }
}

bitflags! {
    pub struct ElfSectionFlags: u64 {
        const WRITABLE = 0x1;
        const ALLOCATED = 0x2;
        const EXECUTABLE = 0x4;
    }
}
