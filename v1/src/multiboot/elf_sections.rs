use super::{Tag, TagTrait, TagType};
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<u32>() + 4 * size_of::<u32>();

#[derive(ptr_meta::Pointee, PartialEq, Eq)]
#[repr(C)]
pub struct ElfSectionsTag {
    typ: u32,
    pub size: u32,
    number_of_sections: u32,
    pub entry_size: u32,
    pub shndx: u32,
    sections: [u8],
}

impl ElfSectionsTag {
    pub fn sections(&self) -> ElfSectionIter {
        let string_section_offset = (self.shndx * self.entry_size) as isize;
        let string_section_ptr =
            unsafe { self.first_section().offset(string_section_offset) as *const _ };
        ElfSectionIter {
            current_section: self.first_section(),
            remaining_sections: self.number_of_sections,
            entry_size: self.entry_size,
            string_section: string_section_ptr,
        }
    }

    fn first_section(&self) -> *const u8 {
        &self.sections[0]
    }
}

impl TagTrait for ElfSectionsTag {
    const ID: TagType = TagType::ElfSections;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

#[derive(Clone)]
pub struct ElfSectionIter {
    current_section: *const u8,
    remaining_sections: u32,
    entry_size: u32,
    string_section: *const u8,
}

impl Iterator for ElfSectionIter {
    type Item = ElfSection;

    fn next(&mut self) -> Option<ElfSection> {
        while self.remaining_sections != 0 {
            let section = ElfSection {
                inner: self.current_section,
                string_section: self.string_section,
                entry_size: self.entry_size,
            };
            self.current_section = unsafe { self.current_section.offset(self.entry_size as isize) };
            self.remaining_sections -= 1;
            if section.is_used() {
                return Some(section);
            }
        }
        None
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElfSection {
    inner: *const u8,
    string_section: *const u8,
    entry_size: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct ElfSectionInner32 {
    name_index: u32,
    typ: u32,
    flags: u32,
    addr: u32,
    offset: u32,
    size: u32,
    link: u32,
    info: u32,
    addralign: u32,
    entry_size: u32,
}

#[derive(Clone, Copy)]
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
        self.get().addr()
    }

    pub fn end_address(&self) -> u64 {
        self.get().addr() + self.get().size()
    }

    pub fn size(&self) -> u64 {
        self.get().size()
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.get().flags())
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ElfSectionFlags::ALLOCATED)
    }

    fn is_used(&self) -> bool {
        self.get().typ() != 0
    }

    fn get(&self) -> &dyn ElfSectionInner {
        match self.entry_size {
            40 => unsafe { &*(self.inner as *const ElfSectionInner32) },
            64 => unsafe { &*(self.inner as *const ElfSectionInner64) },
            s => panic!("Unexpected entry size: {}", s),
        }
    }
}

trait ElfSectionInner {
    fn typ(&self) -> u32;
    fn flags(&self) -> u64;
    fn addr(&self) -> u64;
    fn size(&self) -> u64;
}

impl ElfSectionInner for ElfSectionInner32 {
    fn typ(&self) -> u32 {
        self.typ
    }

    fn flags(&self) -> u64 {
        self.flags.into()
    }

    fn addr(&self) -> u64 {
        self.addr.into()
    }

    fn size(&self) -> u64 {
        self.size.into()
    }
}

impl ElfSectionInner for ElfSectionInner64 {
    fn typ(&self) -> u32 {
        self.typ
    }

    fn flags(&self) -> u64 {
        self.flags
    }

    fn addr(&self) -> u64 {
        self.addr
    }

    fn size(&self) -> u64 {
        self.size
    }
}

bitflags! {
    #[derive(Clone, Copy,  Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct ElfSectionFlags: u64 {
        const WRITABLE = 0x1;
        const ALLOCATED = 0x2;
        const EXECUTABLE = 0x4;
    }
}
