use core::mem::size_of;

use bit_field::BitField as _;
use bitflags::bitflags;
use x86_64::structures::tss::TaskStateSegment;

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
        Self::UserSegment(flags.bits())
    }

    /// segment segment?
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Self {
        let ptr = tss as *const _ as u64;
        let mut low = DescriptorFlags::PRESENT.bits();
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(40..44, 0b1001); // type (0b1001 = available 64-bit tss)
        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));
        Self::SystemSegment(low, high)
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
