use bitflags::bitflags;

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

bitflags! {
    struct DescriptorFlags: u64 {
        const CONFORMING   = 1 << 42;
        const EXECUTABLE   = 1 << 43;
        const USER_SEGMENT = 1 << 44;
        const PRESENT      = 1 << 47;
        const LONG_MODE    = 1 << 53;
    }
}

impl Descriptor {
    pub fn kernel_code_segment() -> Self {
        let flags = DescriptorFlags::USER_SEGMENT
            | DescriptorFlags::PRESENT
            | DescriptorFlags::EXECUTABLE
            | DescriptorFlags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }
}

pub struct Gdt {
    table: [u64; 8],
    next_free: usize,
}

impl Gdt {
    pub fn new() -> Self {
        Self {
            table: [0; 8],
            next_free: 1,
        }
    }
}
