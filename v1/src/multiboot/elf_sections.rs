//! Module for [`ElfSectionsTag`].

use super::{Tag, TagTrait, TagType, TagTypeId};
use core::fmt::Debug;
use core::mem::size_of;

const METADATA_SIZE: usize = size_of::<TagTypeId>() + 4 * size_of::<u32>();

/// This tag contains the section header table from an ELF binary.
// The sections iterator is provided via the [`ElfSectionsTag::sections`]
// method.
#[derive(ptr_meta::Pointee, PartialEq, Eq)]
#[repr(C)]
pub struct ElfSectionsTag {
    typ: TagTypeId,
    pub(crate) size: u32,
    number_of_sections: u32,
    pub(crate) entry_size: u32,
    pub(crate) shndx: u32, // string table
    sections: [u8],
}

impl ElfSectionsTag {
    /// Get an iterator of loaded ELF sections.
    pub(crate) fn sections(&self) -> ElfSectionIter {
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

/// An iterator over some ELF sections.
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

/// A single generic ELF Section.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElfSection {
    inner: *const u8,
    string_section: *const u8,
    entry_size: u32,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
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
    /// Get the section type as a `ElfSectionType` enum variant.
    pub fn section_type(&self) -> ElfSectionType {
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
                log::warn!(
                    "Unknown section type {:x}. Treating as ElfSectionType::Unused",
                    e
                );
                ElfSectionType::Unused
            }
        }
    }

    /// Get the physical start address of the section.
    pub fn start_address(&self) -> u64 {
        self.get().addr()
    }

    /// Get the physical end address of the section.
    ///
    /// This is the same as doing `section.start_address() + section.size()`
    pub fn end_address(&self) -> u64 {
        self.get().addr() + self.get().size()
    }

    /// Get the section's size in bytes.
    pub fn size(&self) -> u64 {
        self.get().size()
    }

    /// Get the section's flags.
    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.get().flags())
    }

    /// Check if the `ALLOCATED` flag is set in the section flags.
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

/// An enum abstraction over raw ELF section types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum ElfSectionType {
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have undefined
    /// values.
    Unused = 0,

    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramSection = 1,

    /// This section holds a linker symbol table.
    LinkerSymbolTable = 2,

    /// The section holds a string table.
    StringTable = 3,

    /// The section holds relocation entries with explicit addends, such as type
    /// Elf32_Rela for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelaRelocation = 4,

    /// The section holds a symbol hash table.
    SymbolHashTable = 5,

    /// The section holds dynamic linking tables.
    DynamicLinkingTable = 6,

    /// This section holds information that marks the file in some way.
    Note = 7,

    /// A section of this type occupies no space in the file but otherwise resembles
    /// `ProgramSection`. Although this section contains no bytes, the
    /// sh_offset member contains the conceptual file offset.
    Uninitialized = 8,

    /// The section holds relocation entries without explicit addends, such as type
    /// Elf32_Rel for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelRelocation = 9,

    /// This section type is reserved but has unspecified semantics.
    Reserved = 10,

    /// This section holds a dynamic loader symbol table.
    DynamicLoaderSymbolTable = 11,

    /// Values in this inclusive range (`[0x6000_0000, 0x6FFF_FFFF)`) are
    /// reserved for environment-specific semantics.
    EnvironmentSpecific = 0x6000_0000,

    /// Values in this inclusive range (`[0x7000_0000, 0x7FFF_FFFF)`) are
    /// reserved for processor-specific semantics.
    ProcessorSpecific = 0x7000_0000,
}

bitflags! {
    /// ELF Section bitflags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct ElfSectionFlags: u64 {
        /// The section contains data that should be writable during program execution.
        const WRITABLE = 0x1;

        /// The section occupies memory during the process execution.
        const ALLOCATED = 0x2;

        /// The section contains executable machine instructions.
        const EXECUTABLE = 0x4;
        // plus environment-specific use at 0x0F000000
        // plus processor-specific use at 0xF0000000
    }
}
