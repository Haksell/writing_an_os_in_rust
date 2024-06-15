use core::marker::PhantomData;
use ptr_meta::Pointee;

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

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tag {
    pub typ: u32,
    pub size: u32,
    // followed by additional, tag specific fields
}

impl Tag {
    fn typ(&self) -> TagType {
        self.typ.into()
    }

    pub fn cast_tag<'a, T: TagTrait + ?Sized + 'a>(&'a self) -> &'a T {
        assert_eq!(self.typ, T::ID.into());
        unsafe { TagTrait::from_base_tag(self) }
    }
}

#[derive(Clone)]
pub struct TagIter<'a> {
    current: *const Tag,
    _mem: PhantomData<&'a ()>,
}

impl<'a> TagIter<'a> {
    pub fn new(mem: &'a [u8]) -> Self {
        assert_eq!(mem.as_ptr().align_offset(8), 0);
        TagIter {
            current: mem.as_ptr().cast(),
            _mem: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        let tag = unsafe { &*self.current };
        match tag.typ() {
            TagType::End => None,
            _ => {
                let ptr_offset = (tag.size as usize + 7) & !7;
                self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };
                Some(tag)
            }
        }
    }
}

pub trait TagTrait: Pointee {
    const ID: TagType;

    fn dst_size(base_tag: &Tag) -> Self::Metadata;

    unsafe fn from_base_tag(tag: &Tag) -> &Self {
        let ptr = core::ptr::addr_of!(*tag);
        let ptr = ptr_meta::from_raw_parts(ptr.cast(), Self::dst_size(tag));
        &*ptr
    }
}
