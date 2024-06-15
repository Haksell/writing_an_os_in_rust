use super::{Tag, TagTrait, TagType, TagTypeId};

/// This tag contains a copy of RSDP as defined per ACPI 1.0 specification.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct RsdpV1Tag {
    typ: TagTypeId,
    size: u32,
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32, // This is the PHYSICAL address of the RSDT
}

impl TagTrait for RsdpV1Tag {
    const ID: TagType = TagType::AcpiV1;

    fn dst_size(_base_tag: &Tag) {}
}

/// This tag contains a copy of RSDP as defined per ACPI 2.0 or later specification.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct RsdpV2Tag {
    typ: TagTypeId,
    size: u32,
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    length: u32,
    xsdt_address: u64,
    // This is the PHYSICAL address of the XSDT
    ext_checksum: u8,
    _reserved: [u8; 3],
}

impl TagTrait for RsdpV2Tag {
    const ID: TagType = TagType::AcpiV2;

    fn dst_size(_base_tag: &Tag) {}
}
