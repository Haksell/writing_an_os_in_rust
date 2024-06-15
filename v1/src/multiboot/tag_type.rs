//! Module for tag types.
//!
//! The relevant exports of this module are [`TagTypeId`] and [`TagType`].

use core::fmt::{Debug, Formatter};
use core::hash::Hash;

/// Serialized form of [`TagType`] that matches the binary representation
/// (`u32`). The abstraction corresponds to the `typ`/`type` field of a
/// Multiboot2 [`Tag`]. This type can easily be created from or converted to
/// [`TagType`].
///
/// [`Tag`]: crate::Tag
#[repr(transparent)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct TagTypeId(u32);

impl TagTypeId {
    /// Constructor.
    pub fn new(val: u32) -> Self {
        Self(val)
    }
}

impl Debug for TagTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let tag_type = TagType::from(*self);
        Debug::fmt(&tag_type, f)
    }
}

/// Higher level abstraction for [`TagTypeId`] that assigns each possible value
/// to a specific semantic according to the specification. Additionally, it
/// allows to use the [`TagType::Custom`] variant. It is **not binary compatible**
/// with [`TagTypeId`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagType {
    /// Tag `0`: Marks the end of the tags.
    End,
    /// Tag `1`: Additional command line string.
    /// For example `''` or `'--my-custom-option foo --provided by_grub`, if
    /// your GRUB config contains `multiboot2 /boot/multiboot2-binary.elf --my-custom-option foo --provided by_grub`
    Cmdline,
    /// Tag `2`: Name of the bootloader, e.g. 'GRUB 2.04-1ubuntu44.2'
    BootLoaderName,
    /// Tag `3`: Additional Multiboot modules, which are BLOBs provided in
    /// memory. For example an initial ram disk with essential drivers.
    Module,
    /// Tag `4`: ‘mem_lower’ and ‘mem_upper’ indicate the amount of lower and
    /// upper memory, respectively, in kilobytes. Lower memory starts at
    /// address 0, and upper memory starts at address 1 megabyte. The maximum
    /// possible value for lower memory is 640 kilobytes. The value returned
    /// for upper memory is maximally the address of the first upper memory
    /// hole minus 1 megabyte. It is not guaranteed to be this value.
    ///
    /// This tag may not be provided by some boot loaders on EFI platforms if
    /// EFI boot services are enabled and available for the loaded image (EFI
    /// boot services not terminated tag exists in Multiboot2 information
    /// structure).
    BasicMeminfo,
    /// Tag `5`: This tag indicates which BIOS disk device the boot loader
    /// loaded the OS image from. If the OS image was not loaded from a BIOS
    /// disk, then this tag must not be present. The operating system may use
    /// this field as a hint for determining its own root device, but is not
    /// required to.
    Bootdev,
    /// Tag `6`: Memory map. The map provided is guaranteed to list all
    /// standard RAM that should be available for normal use. This type however
    /// includes the regions occupied by kernel, mbi, segments and modules.
    /// Kernel must take care not to overwrite these regions.
    ///
    /// This tag may not be provided by some boot loaders on EFI platforms if
    /// EFI boot services are enabled and available for the loaded image (EFI
    /// boot services not terminated tag exists in Multiboot2 information
    /// structure).
    Mmap,
    /// Tag `7`: Contains the VBE control information returned by the VBE
    /// Function `0x00` and VBE mode information returned by the VBE Function
    /// `0x01`, respectively. Note that VBE 3.0 defines another protected mode
    /// interface which is incompatible with the old one. If you want to use
    /// the new protected mode interface, you will have to find the table
    /// yourself.
    Vbe,
    /// Tag `8`: Framebuffer.
    Framebuffer,
    /// Tag `9`: This tag contains section header table from an ELF kernel, the
    /// size of each entry, number of entries, and the string table used as the
    /// index of names. They correspond to the `shdr_*` entries (`shdr_num`,
    /// etc.) in the Executable and Linkable Format (ELF) specification in the
    /// program header.
    ElfSections,
    /// Tag `10`: APM table. See Advanced Power Management (APM) BIOS Interface
    /// Specification, for more information.
    Apm,
    /// Tag `11`: This tag contains pointer to i386 EFI system table.
    Efi32,
    /// Tag `21`: This tag contains pointer to amd64 EFI system table.
    Efi64,
    /// Tag `13`: This tag contains a copy of SMBIOS tables as well as their
    /// version.
    Smbios,
    /// Tag `14`: Also called "AcpiOld" in other multiboot2 implementations.
    AcpiV1,
    /// Tag `15`: Refers to version 2 and later of Acpi.
    /// Also called "AcpiNew" in other multiboot2 implementations.
    AcpiV2,
    /// Tag `16`: This tag contains network information in the format specified
    /// as DHCP. It may be either a real DHCP reply or just the configuration
    /// info in the same format. This tag appears once
    /// per card.
    Network,
    /// Tag `17`: This tag contains EFI memory map as per EFI specification.
    /// This tag may not be provided by some boot loaders on EFI platforms if
    /// EFI boot services are enabled and available for the loaded image (EFI
    /// boot services not terminated tag exists in Multiboot2 information
    /// structure).
    EfiMmap,
    /// Tag `18`: This tag indicates ExitBootServices wasn't called.
    EfiBs,
    /// Tag `19`: This tag contains pointer to EFI i386 image handle. Usually
    /// it is boot loader image handle.
    Efi32Ih,
    /// Tag `20`: This tag contains pointer to EFI amd64 image handle. Usually
    /// it is boot loader image handle.
    Efi64Ih,
    /// Tag `21`: This tag contains image load base physical address. The spec
    /// tells *"It is provided only if image has relocatable header tag."* but
    /// experience showed that this is not true for at least GRUB 2.
    LoadBaseAddr,
    /// Custom tag types `> 21`. The Multiboot2 spec doesn't explicitly allow
    /// or disallow them. Bootloader and OS developers are free to use custom
    /// tags.
    Custom(u32),
}

impl TagType {
    /// Convenient wrapper to get the underlying `u32` representation of the tag.
    pub fn val(&self) -> u32 {
        u32::from(*self)
    }
}

/// Relevant `From`-implementations for conversions between `u32`, [´TagTypeId´]
/// and [´TagType´].
mod primitive_conversion_impls {
    use super::*;

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
}

/// `From`-implementations for conversions between [´TagTypeId´] and [´TagType´].
mod intermediate_conversion_impls {
    use super::*;

    impl From<TagTypeId> for TagType {
        fn from(value: TagTypeId) -> Self {
            let value = u32::from(value);
            TagType::from(value)
        }
    }

    impl From<TagType> for TagTypeId {
        fn from(value: TagType) -> Self {
            let value = u32::from(value);
            TagTypeId::from(value)
        }
    }
}

/// Implements `partial_eq` between [´TagTypeId´] and [´TagType´]. Two values
/// are equal if their `u32` representation is equal. Additionally, `u32` can
/// be compared with [´TagTypeId´].
mod partial_eq_impls {
    use super::*;

    impl PartialEq<TagTypeId> for TagType {
        fn eq(&self, other: &TagTypeId) -> bool {
            let this = u32::from(*self);
            let that = u32::from(*other);
            this == that
        }
    }

    // each compare/equal direction must be implemented manually
    impl PartialEq<TagType> for TagTypeId {
        fn eq(&self, other: &TagType) -> bool {
            other.eq(self)
        }
    }

    impl PartialEq<u32> for TagTypeId {
        fn eq(&self, other: &u32) -> bool {
            let this = u32::from(*self);
            this == *other
        }
    }

    impl PartialEq<TagTypeId> for u32 {
        fn eq(&self, other: &TagTypeId) -> bool {
            other.eq(self)
        }
    }

    impl PartialEq<u32> for TagType {
        fn eq(&self, other: &u32) -> bool {
            let this = u32::from(*self);
            this == *other
        }
    }

    impl PartialEq<TagType> for u32 {
        fn eq(&self, other: &TagType) -> bool {
            other.eq(self)
        }
    }
}
