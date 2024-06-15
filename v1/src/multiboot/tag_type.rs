use core::hash::Hash;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagType {
    End,
    Mmap,
    ElfSections,
    Custom(u32),
}

impl From<u32> for TagType {
    fn from(value: u32) -> Self {
        match value {
            0 => TagType::End,
            6 => TagType::Mmap,
            9 => TagType::ElfSections,
            c => TagType::Custom(c),
        }
    }
}

impl From<TagType> for u32 {
    fn from(value: TagType) -> Self {
        match value {
            TagType::End => 0,
            TagType::Mmap => 6,
            TagType::ElfSections => 9,
            TagType::Custom(c) => c,
        }
    }
}

impl PartialEq<u32> for TagType {
    fn eq(&self, other: &u32) -> bool {
        u32::from(*self) == u32::from(*other)
    }
}

impl PartialEq<TagType> for u32 {
    fn eq(&self, other: &TagType) -> bool {
        other.eq(self)
    }
}
