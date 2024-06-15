//! Module for [`FramebufferTag`].

use super::{Tag, TagTrait, TagType, TagTypeId};
use core::fmt::Debug;
use core::mem::size_of;
use core::slice;
use derive_more::Display;

/// Helper struct to read bytes from a raw pointer and increase the pointer
/// automatically.
struct Reader {
    ptr: *const u8,
    off: usize,
}

impl Reader {
    fn new<T>(ptr: *const T) -> Reader {
        Reader {
            ptr: ptr as *const u8,
            off: 0,
        }
    }

    fn read_u8(&mut self) -> u8 {
        self.off += 1;
        unsafe { *self.ptr.add(self.off - 1) }
    }

    fn read_u16(&mut self) -> u16 {
        self.read_u8() as u16 | (self.read_u8() as u16) << 8
    }

    fn read_u32(&mut self) -> u32 {
        self.read_u16() as u32 | (self.read_u16() as u32) << 16
    }

    fn current_address(&self) -> usize {
        unsafe { self.ptr.add(self.off) as usize }
    }
}

const METADATA_SIZE: usize = size_of::<TagTypeId>()
    + 4 * size_of::<u32>()
    + size_of::<u64>()
    + size_of::<u16>()
    + 2 * size_of::<u8>();

/// The VBE Framebuffer information tag.
#[derive(ptr_meta::Pointee, Eq)]
#[repr(C)]
pub struct FramebufferTag {
    typ: TagTypeId,
    size: u32,

    /// Contains framebuffer physical address.
    ///
    /// This field is 64-bit wide but bootloader should set it under 4GiB if
    /// possible for compatibility with payloads which aren’t aware of PAE or
    /// amd64.
    address: u64,

    /// Contains the pitch in bytes.
    pitch: u32,

    /// Contains framebuffer width in pixels.
    width: u32,

    /// Contains framebuffer height in pixels.
    height: u32,

    /// Contains number of bits per pixel.
    bpp: u8,

    /// The type of framebuffer, one of: `Indexed`, `RGB` or `Text`.
    type_no: u8,

    // In the multiboot spec, it has this listed as a u8 _NOT_ a u16.
    // Reading the GRUB2 source code reveals it is in fact a u16.
    _reserved: u16,

    buffer: [u8],
}

impl TagTrait for FramebufferTag {
    const ID: TagType = TagType::Framebuffer;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

impl PartialEq for FramebufferTag {
    fn eq(&self, other: &Self) -> bool {
        ({ self.typ } == { other.typ }
            && { self.size } == { other.size }
            && { self.address } == { other.address }
            && { self.pitch } == { other.pitch }
            && { self.width } == { other.width }
            && { self.height } == { other.height }
            && { self.bpp } == { other.bpp }
            && { self.type_no } == { other.type_no }
            && self.buffer == other.buffer)
    }
}

/// Helper struct for [`FramebufferType`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
enum FramebufferTypeId {
    Indexed = 0,
    RGB = 1,
    Text = 2,
    // spec says: there may be more variants in the future
}

impl TryFrom<u8> for FramebufferTypeId {
    type Error = UnknownFramebufferType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Indexed),
            1 => Ok(Self::RGB),
            2 => Ok(Self::Text),
            val => Err(UnknownFramebufferType(val)),
        }
    }
}

/// The type of framebuffer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FramebufferType<'a> {
    /// Indexed color.
    Indexed {
        #[allow(missing_docs)]
        palette: &'a [FramebufferColor],
    },

    /// Direct RGB color.
    #[allow(missing_docs)]
    #[allow(clippy::upper_case_acronyms)]
    RGB {
        red: FramebufferField,
        green: FramebufferField,
        blue: FramebufferField,
    },

    /// EGA Text.
    ///
    /// In this case the framebuffer width and height are expressed in
    /// characters and not in pixels.
    ///
    /// The bpp is equal 16 (16 bits per character) and pitch is expressed in bytes per text line.
    Text,
}

/// An RGB color type field.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct FramebufferField {
    /// Color field position.
    pub position: u8,

    /// Color mask size.
    pub size: u8,
}

/// A framebuffer color descriptor in the palette.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)] // no align(8) here is correct
pub struct FramebufferColor {
    /// The Red component of the color.
    pub red: u8,

    /// The Green component of the color.
    pub green: u8,

    /// The Blue component of the color.
    pub blue: u8,
}

/// Error when an unknown [`FramebufferTypeId`] is found.
#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
#[display(fmt = "Unknown framebuffer type {}", _0)]
pub struct UnknownFramebufferType(u8);
