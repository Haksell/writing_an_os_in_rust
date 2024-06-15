use super::{Tag, TagTrait, TagType, TagTypeId};
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<TagTypeId>() + 4 * size_of::<u32>();

#[derive(ptr_meta::Pointee, PartialEq, Eq)]
#[repr(C)]
pub struct ElfSectionsTag {
    typ: TagTypeId,
    pub size: u32,
    number_of_sections: u32,
    pub entry_size: u32,
    pub shndx: u32, // string table
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
        &(self.sections[0]) as *const _
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

            if section.section_type() != ElfSectionType::Unused {
                return Some(section);
            }
        }
        None
    }
}

impl Default for ElfSectionIter {
    fn default() -> Self {
        Self {
            current_section: core::ptr::null(),
            remaining_sections: 0,
            entry_size: 0,
            string_section: core::ptr::null(),
        }
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
    fn section_type(&self) -> ElfSectionType {
        match self.get().typ() {
            0 => ElfSectionType::Unused,
            1 => ElfSectionType::ProgramSection,
            2 => ElfSectionType::LinkerSymbolTable,
            3 => ElfSectionType::StringTable,
            4 => ElfSectionType::RelaRelocation,
            5 => ElfSectionType::SymbolHashTable,
            6 => ElfSectionType::DynamicLinkingTable,
            7 => ElfSectionType::Note,
            8 => ElfSectionType::Uninitialized,
            9 => ElfSectionType::RelRelocation,
            10 => ElfSectionType::Reserved,
            11 => ElfSectionType::DynamicLoaderSymbolTable,
            0x6000_0000..=0x6FFF_FFFF => ElfSectionType::EnvironmentSpecific,
            0x7000_0000..=0x7FFF_FFFF => ElfSectionType::ProcessorSpecific,
            e => {
                println!(
                    "Unknown section type {:x}. Treating as ElfSectionType::Unused",
                    e
                );
                ElfSectionType::Unused
            }
        }
    }
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum ElfSectionType {
    Unused = 0,
    ProgramSection = 1,
    LinkerSymbolTable = 2,
    StringTable = 3,
    RelaRelocation = 4,
    SymbolHashTable = 5,
    DynamicLinkingTable = 6,
    Note = 7,
    Uninitialized = 8,
    RelRelocation = 9,
    Reserved = 10,
    DynamicLoaderSymbolTable = 11,
    EnvironmentSpecific = 0x6000_0000,
    ProcessorSpecific = 0x7000_0000,
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
