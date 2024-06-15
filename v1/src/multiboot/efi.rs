use super::TagTypeId;
use super::{Tag, TagTrait, TagType};

/// EFI system table in 32 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFISdt32Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u32,
}

impl EFISdt32Tag {}

impl TagTrait for EFISdt32Tag {
    const ID: TagType = TagType::Efi32;

    fn dst_size(_base_tag: &Tag) {}
}

/// EFI system table in 64 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFISdt64Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u64,
}

impl TagTrait for EFISdt64Tag {
    const ID: TagType = TagType::Efi64;

    fn dst_size(_base_tag: &Tag) {}
}

/// Tag that contains the pointer to the boot loader's UEFI image handle
/// (32-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIImageHandle32Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u32,
}

impl TagTrait for EFIImageHandle32Tag {
    const ID: TagType = TagType::Efi32Ih;

    fn dst_size(_base_tag: &Tag) {}
}

/// Tag that contains the pointer to the boot loader's UEFI image handle
/// (64-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIImageHandle64Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u64,
}

impl TagTrait for EFIImageHandle64Tag {
    const ID: TagType = TagType::Efi64Ih;

    fn dst_size(_base_tag: &Tag) {}
}

/// EFI ExitBootServices was not called tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIBootServicesNotExitedTag {
    typ: TagTypeId,
    size: u32,
}

impl TagTrait for EFIBootServicesNotExitedTag {
    const ID: TagType = TagType::EfiBs;

    fn dst_size(_base_tag: &Tag) {}
}
