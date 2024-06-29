use bitflags::bitflags;
use core::arch::asm;

#[derive(Debug)]
pub struct Cr0;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Cr0Flags: u64 {
        const PROTECTED_MODE_ENABLE = 1;
        const MONITOR_COPROCESSOR = 1 << 1;
        const EMULATE_COPROCESSOR = 1 << 2;
        const TASK_SWITCHED = 1 << 3;
        const EXTENSION_TYPE = 1 << 4;
        const NUMERIC_ERROR = 1 << 5;
        const WRITE_PROTECT = 1 << 16;
        const ALIGNMENT_MASK = 1 << 18;
        const NOT_WRITE_THROUGH = 1 << 29;
        const CACHE_DISABLE = 1 << 30;
        const PAGING = 1 << 31;
    }
}

impl Cr0 {
    #[inline]
    pub fn read() -> Cr0Flags {
        Cr0Flags::from_bits_truncate(Self::read_raw())
    }

    #[inline]
    pub fn read_raw() -> u64 {
        let value: u64;

        unsafe {
            asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
        }

        value
    }

    #[inline]
    pub unsafe fn write(flags: Cr0Flags) {
        let old_value = Self::read_raw();
        let reserved = old_value & !(Cr0Flags::all().bits());
        let new_value = reserved | flags.bits();

        unsafe {
            Self::write_raw(new_value);
        }
    }

    #[inline]
    pub unsafe fn write_raw(value: u64) {
        unsafe {
            asm!("mov cr0, {}", in(reg) value, options(nostack, preserves_flags));
        }
    }
}
