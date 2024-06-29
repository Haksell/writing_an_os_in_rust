use bit_field::BitField as _;
use bitflags::bitflags;
use core::mem::size_of;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::DescriptorTablePointer;
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;

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

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(low, high) => {
                let index = self.push(low);
                self.push(high);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    fn push(&mut self, value: u64) -> usize {
        assert!(self.next_free < self.table.len(), "GDT full");
        let index = self.next_free;
        self.table[index] = value;
        self.next_free += 1;
        index
    }

    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
            base: VirtAddr::new(self.table.as_ptr() as u64),
        };
        unsafe { crate::asm::lgdt(&ptr) };
    }
}
