#![cfg_attr(feature = "unstable", feature(error_in_core))]
#![deny(missing_debug_implementations)]
// --- BEGIN STYLE CHECKS ---
// These checks are optional in CI for PRs, as discussed in
// https://github.com/rust-osdev/multiboot2/pull/92
#![deny(clippy::all)]
#![deny(rustdoc::all)]
#![allow(rustdoc::private_doc_tests)]

// this crate can use std in tests only
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

mod boot_loader_name;
mod command_line;
mod efi;
mod elf_sections;
mod end;
mod framebuffer;
mod image_load_addr;
mod memory_map;
mod module;
mod rsdp;
mod smbios;
mod tag;
mod tag_trait;
mod tag_type;
mod vbe_info;

pub use boot_loader_name::BootLoaderNameTag;
pub use command_line::CommandLineTag;
pub use efi::{
    EFIBootServicesNotExitedTag, EFIImageHandle32Tag, EFIImageHandle64Tag, EFISdt32Tag, EFISdt64Tag,
};
pub use elf_sections::{ElfSection, ElfSectionFlags, ElfSectionIter, ElfSectionsTag};
pub use end::EndTag;
pub use framebuffer::FramebufferTag;
pub use image_load_addr::ImageLoadPhysAddrTag;
pub use memory_map::{BasicMemoryInfoTag, EFIMemoryMapTag, MemoryArea, MemoryMapTag};
pub use module::ModuleIter;
pub use rsdp::{RsdpV1Tag, RsdpV2Tag};
pub use smbios::SmbiosTag;
pub use tag::Tag;
pub use tag_trait::TagTrait;
pub use tag_type::{TagType, TagTypeId};
pub use vbe_info::VBEInfoTag;

use core::fmt;
use core::mem::size_of;
use derive_more::Display;
// Must be public so that custom tags can be DSTs.
use framebuffer::UnknownFramebufferType;
use tag::TagIter;

/// Error type that describes errors while loading/parsing a multiboot2 information structure
/// from a given address.
#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MbiLoadError {
    /// The address is invalid. Make sure that the address is 8-byte aligned,
    /// according to the spec.
    #[display(fmt = "The address is invalid")]
    IllegalAddress,
    /// The total size of the multiboot2 information structure must be not zero
    /// and a multiple of 8.
    #[display(fmt = "The size of the MBI is unexpected")]
    IllegalTotalSize(u32),
    /// Missing end tag. Each multiboot2 boot information requires to have an
    /// end tag.
    #[display(fmt = "There is no end tag")]
    NoEndTag,
}

/// The basic header of a boot information.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootInformationHeader {
    // size is multiple of 8
    pub total_size: u32,
    _reserved: u32,
    // Followed by the boot information tags.
}

#[derive(ptr_meta::Pointee)]
#[repr(C)]
struct BootInformationInner {
    header: BootInformationHeader,
    tags: [u8],
}

impl BootInformationInner {
    /// Checks if the MBI has a valid end tag by checking the end of the mbi's
    /// bytes.
    fn has_valid_end_tag(&self) -> bool {
        let end_tag_prototype = EndTag::default();

        let self_ptr = unsafe { self.tags.as_ptr().sub(size_of::<BootInformationHeader>()) };

        let end_tag_ptr = unsafe {
            self_ptr
                .add(self.header.total_size as usize)
                .sub(size_of::<EndTag>())
        };
        let end_tag = unsafe { &*(end_tag_ptr as *const EndTag) };

        end_tag.typ == end_tag_prototype.typ && end_tag.size == end_tag_prototype.size
    }
}

/// A Multiboot 2 Boot Information (MBI) accessor.
#[repr(transparent)]
pub struct BootInformation<'a>(&'a BootInformationInner);

impl<'a> BootInformation<'a> {
    pub unsafe fn load(ptr: *const BootInformationHeader) -> Result<Self, MbiLoadError> {
        // null or not aligned
        if ptr.is_null() || ptr.align_offset(8) != 0 {
            return Err(MbiLoadError::IllegalAddress);
        }

        // mbi: reference to basic header
        let mbi = &*ptr;

        // Check if total size is not 0 and a multiple of 8.
        if mbi.total_size == 0 || mbi.total_size & 0b111 != 0 {
            return Err(MbiLoadError::IllegalTotalSize(mbi.total_size));
        }

        let slice_size = mbi.total_size as usize - size_of::<BootInformationHeader>();
        // mbi: reference to full mbi
        let mbi = ptr_meta::from_raw_parts::<BootInformationInner>(ptr.cast(), slice_size);
        let mbi = &*mbi;

        if !mbi.has_valid_end_tag() {
            return Err(MbiLoadError::NoEndTag);
        }

        Ok(Self(mbi))
    }

    /// Get the start address of the boot info.
    pub fn start_address(&self) -> usize {
        self.as_ptr() as usize
    }

    /// Get the start address of the boot info as pointer.
    pub fn as_ptr(&self) -> *const () {
        core::ptr::addr_of!(*self.0).cast()
    }

    /// Get the end address of the boot info.
    ///
    /// This is the same as doing:
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// let end_addr = boot_info.start_address() + boot_info.total_size();
    /// ```
    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    /// Get the total size of the boot info struct.
    pub fn total_size(&self) -> usize {
        self.0.header.total_size as usize
    }

    // ######################################################
    // ### BEGIN OF TAG GETTERS (in alphabetical order)

    /*fn apm(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the basic memory info tag.
    pub fn basic_memory_info_tag(&self) -> Option<&BasicMemoryInfoTag> {
        self.get_tag::<BasicMemoryInfoTag>()
    }

    /// Search for the BootLoader name tag.
    pub fn boot_loader_name_tag(&self) -> Option<&BootLoaderNameTag> {
        self.get_tag::<BootLoaderNameTag>()
    }

    /*fn bootdev(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the Command line tag.
    pub fn command_line_tag(&self) -> Option<&CommandLineTag> {
        self.get_tag::<CommandLineTag>()
    }

    /// Search for the EFI boot services not exited tag.
    pub fn efi_bs_not_exited_tag(&self) -> Option<&EFIBootServicesNotExitedTag> {
        self.get_tag::<EFIBootServicesNotExitedTag>()
    }

    /// Search for the EFI Memory map tag, if the boot services were exited.
    /// Otherwise, if the [`TagType::EfiBs`] tag is present, this returns `None`
    /// as it is strictly recommended to get the memory map from the `uefi`
    /// services.
    pub fn efi_memory_map_tag(&self) -> Option<&EFIMemoryMapTag> {
        // If the EFIBootServicesNotExited is present, then we should not use
        // the memory map, as it could still be in use.
        match self.get_tag::<EFIBootServicesNotExitedTag>() {
            Some(_tag) => {
                log::debug!("The EFI memory map is present but the UEFI Boot Services Not Existed Tag is present. Returning None.");
                None
            }
            None => self.get_tag::<EFIMemoryMapTag>(),
        }
    }

    /// Search for the EFI 32-bit SDT tag.
    pub fn efi_sdt32_tag(&self) -> Option<&EFISdt32Tag> {
        self.get_tag::<EFISdt32Tag>()
    }

    /// Search for the EFI 64-bit SDT tag.
    pub fn efi_sdt64_tag(&self) -> Option<&EFISdt64Tag> {
        self.get_tag::<EFISdt64Tag>()
    }

    /// Search for the EFI 32-bit image handle pointer tag.
    pub fn efi_ih32_tag(&self) -> Option<&EFIImageHandle32Tag> {
        self.get_tag::<EFIImageHandle32Tag>()
    }

    /// Search for the EFI 64-bit image handle pointer tag.
    pub fn efi_ih64_tag(&self) -> Option<&EFIImageHandle64Tag> {
        self.get_tag::<EFIImageHandle64Tag>()
    }

    /// Returns an [`ElfSectionIter`] iterator over the ELF Sections, if the
    /// [`ElfSectionsTag`] is present.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// if let Some(sections) = boot_info.elf_sections() {
    ///     let mut total = 0;
    ///     for section in sections {
    ///         println!("Section: {:?}", section);
    ///         total += 1;
    ///     }
    /// }
    /// ```
    pub fn elf_sections(&self) -> Option<ElfSectionIter> {
        let tag = self.get_tag::<ElfSectionsTag>();
        tag.map(|t| {
            assert!((t.entry_size * t.shndx) <= t.size);
            t.sections()
        })
    }

    /// Search for the VBE framebuffer tag. The result is `Some(Err(e))`, if the
    /// framebuffer type is unknown, while the framebuffer tag is present.
    pub fn framebuffer_tag(&self) -> Option<Result<&FramebufferTag, UnknownFramebufferType>> {
        self.get_tag::<FramebufferTag>()
            .map(|tag| match tag.buffer_type() {
                Ok(_) => Ok(tag),
                Err(e) => Err(e),
            })
    }

    /// Search for the Image Load Base Physical Address tag.
    pub fn load_base_addr_tag(&self) -> Option<&ImageLoadPhysAddrTag> {
        self.get_tag::<ImageLoadPhysAddrTag>()
    }

    /// Search for the Memory map tag.
    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag::<MemoryMapTag>()
    }

    /// Get an iterator of all module tags.
    pub fn module_tags(&self) -> ModuleIter {
        module::module_iter(self.tags())
    }

    /*fn network_tag(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the (ACPI 1.0) RSDP tag.
    pub fn rsdp_v1_tag(&self) -> Option<&RsdpV1Tag> {
        self.get_tag::<RsdpV1Tag>()
    }

    /// Search for the (ACPI 2.0 or later) RSDP tag.
    pub fn rsdp_v2_tag(&self) -> Option<&RsdpV2Tag> {
        self.get_tag::<RsdpV2Tag>()
    }

    /// Search for the SMBIOS tag.
    pub fn smbios_tag(&self) -> Option<&SmbiosTag> {
        self.get_tag::<SmbiosTag>()
    }

    /// Search for the VBE information tag.
    pub fn vbe_info_tag(&self) -> Option<&VBEInfoTag> {
        self.get_tag::<VBEInfoTag>()
    }

    pub fn get_tag<TagT: TagTrait + ?Sized + 'a>(&'a self) -> Option<&'a TagT> {
        self.tags()
            .find(|tag| tag.typ == TagT::ID)
            .map(|tag| tag.cast_tag::<TagT>())
    }

    /// Returns an iterator over all tags.
    fn tags(&self) -> TagIter {
        TagIter::new(&self.0.tags)
    }
}

impl fmt::Debug for BootInformation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /// Limit how many Elf-Sections should be debug-formatted.
        /// Can be thousands of sections for a Rust binary => this is useless output.
        /// If the user really wants this, they should debug-format the field directly.
        const ELF_SECTIONS_LIMIT: usize = 7;

        let mut debug = f.debug_struct("Multiboot2BootInformation");
        debug
            .field("start_address", &self.start_address())
            .field("end_address", &self.end_address())
            .field("total_size", &self.total_size())
            // now tags in alphabetical order
            .field("basic_memory_info", &(self.basic_memory_info_tag()))
            .field("boot_loader_name", &self.boot_loader_name_tag())
            // .field("bootdev", &self.bootdev_tag())
            .field("command_line", &self.command_line_tag())
            .field("efi_bs_not_exited", &self.efi_bs_not_exited_tag())
            .field("efi_memory_map", &self.efi_memory_map_tag())
            .field("efi_sdt32", &self.efi_sdt32_tag())
            .field("efi_sdt64", &self.efi_sdt64_tag())
            .field("efi_ih32", &self.efi_ih32_tag())
            .field("efi_ih64", &self.efi_ih64_tag());

        // usually this is REALLY big (thousands of tags) => skip it here
        {
            let elf_sections_tag_entries_count =
                self.elf_sections().map(|x| x.count()).unwrap_or(0);

            if elf_sections_tag_entries_count > ELF_SECTIONS_LIMIT {
                debug.field("elf_sections (count)", &elf_sections_tag_entries_count);
            } else {
                debug.field("elf_sections", &self.elf_sections().unwrap_or_default());
            }
        }

        debug
            .field("framebuffer", &self.framebuffer_tag())
            .field("load_base_addr", &self.load_base_addr_tag())
            .field("memory_map", &self.memory_map_tag())
            .field("modules", &self.module_tags())
            // .field("network", &self.network_tag())
            .field("rsdp_v1", &self.rsdp_v1_tag())
            .field("rsdp_v2", &self.rsdp_v2_tag())
            .field("smbios_tag", &self.smbios_tag())
            .field("vbe_info_tag", &self.vbe_info_tag())
            .finish()
    }
}
