use core::hash::Hash;
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct TagTypeId(u32);

impl TagTypeId {
    pub fn new(val: u32) -> Self {
        Self(val)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagType {
    End,
    Cmdline,
    BootLoaderName,
    Module,
    BasicMeminfo,
    Bootdev,
    Mmap,
    Vbe,
    Framebuffer,
    ElfSections,
    Apm,
    Efi32,
    Efi64,
    Smbios,
    AcpiV1,
    AcpiV2,
    Network,
    EfiMmap,
    EfiBs,
    Efi32Ih,
    Efi64Ih,
    LoadBaseAddr,
    Custom(u32),
}

impl From<u32> for TagTypeId {
    fn from(value: u32) -> Self {
        // SAFETY: the type has repr(transparent)
        unsafe { core::mem::transmute(value) }
    }
}

impl From<TagTypeId> for u32 {
    fn from(value: TagTypeId) -> Self {
        value.0 as _
    }
}

impl From<u32> for TagType {
    fn from(value: u32) -> Self {
        match value {
            0 => TagType::End,
            1 => TagType::Cmdline,
            2 => TagType::BootLoaderName,
            3 => TagType::Module,
            4 => TagType::BasicMeminfo,
            5 => TagType::Bootdev,
            6 => TagType::Mmap,
            7 => TagType::Vbe,
            8 => TagType::Framebuffer,
            9 => TagType::ElfSections,
            10 => TagType::Apm,
            11 => TagType::Efi32,
            12 => TagType::Efi64,
            13 => TagType::Smbios,
            14 => TagType::AcpiV1,
            15 => TagType::AcpiV2,
            16 => TagType::Network,
            17 => TagType::EfiMmap,
            18 => TagType::EfiBs,
            19 => TagType::Efi32Ih,
            20 => TagType::Efi64Ih,
            21 => TagType::LoadBaseAddr,
            c => TagType::Custom(c),
        }
    }
}

impl From<TagType> for u32 {
    fn from(value: TagType) -> Self {
        match value {
            TagType::End => 0,
            TagType::Cmdline => 1,
            TagType::BootLoaderName => 2,
            TagType::Module => 3,
            TagType::BasicMeminfo => 4,
            TagType::Bootdev => 5,
            TagType::Mmap => 6,
            TagType::Vbe => 7,
            TagType::Framebuffer => 8,
            TagType::ElfSections => 9,
            TagType::Apm => 10,
            TagType::Efi32 => 11,
            TagType::Efi64 => 12,
            TagType::Smbios => 13,
            TagType::AcpiV1 => 14,
            TagType::AcpiV2 => 15,
            TagType::Network => 16,
            TagType::EfiMmap => 17,
            TagType::EfiBs => 18,
            TagType::Efi32Ih => 19,
            TagType::Efi64Ih => 20,
            TagType::LoadBaseAddr => 21,
            TagType::Custom(c) => c,
        }
    }
}

impl From<TagTypeId> for TagType {
    fn from(value: TagTypeId) -> Self {
        let value = u32::from(value);
        TagType::from(value)
    }
}

impl From<TagType> for TagTypeId {
    fn from(value: TagType) -> Self {
        TagTypeId::from(u32::from(value))
    }
}

impl PartialEq<TagTypeId> for TagType {
    fn eq(&self, other: &TagTypeId) -> bool {
        u32::from(*self) == u32::from(*other)
    }
}

impl PartialEq<TagType> for TagTypeId {
    fn eq(&self, other: &TagType) -> bool {
        other.eq(self)
    }
}
