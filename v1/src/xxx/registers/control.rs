use bitflags::bitflags;

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

#[derive(Debug)]
pub struct Cr3;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Cr3Flags: u64 {
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Cr4Flags: u64 {
        const VIRTUAL_8086_MODE_EXTENSIONS = 1;
        const PROTECTED_MODE_VIRTUAL_INTERRUPTS = 1 << 1;
        const TIMESTAMP_DISABLE = 1 << 2;
        const DEBUGGING_EXTENSIONS = 1 << 3;
        const PAGE_SIZE_EXTENSION = 1 << 4;
        const PHYSICAL_ADDRESS_EXTENSION = 1 << 5;
        const MACHINE_CHECK_EXCEPTION = 1 << 6;
        const PAGE_GLOBAL = 1 << 7;
        const PERFORMANCE_MONITOR_COUNTER = 1 << 8;
        const OSFXSR = 1 << 9;
        const OSXMMEXCPT_ENABLE = 1 << 10;
        const USER_MODE_INSTRUCTION_PREVENTION = 1 << 11;
        const L5_PAGING = 1 << 12;
        const VIRTUAL_MACHINE_EXTENSIONS = 1 << 13;
        const SAFER_MODE_EXTENSIONS = 1 << 14;
        const FSGSBASE = 1 << 16;
        const PCID = 1 << 17;
        const OSXSAVE = 1 << 18;
        const KEY_LOCKER = 1 << 19;
        const SUPERVISOR_MODE_EXECUTION_PROTECTION = 1 << 20;
        const SUPERVISOR_MODE_ACCESS_PREVENTION = 1 << 21;
        const PROTECTION_KEY_USER = 1 << 22;
        const CONTROL_FLOW_ENFORCEMENT = 1 << 23;
        const PROTECTION_KEY_SUPERVISOR = 1 << 24;
    }
}

mod x86_64 {
    use super::*;
    use crate::xxx::{structures::paging::PhysFrame, PhysAddr};
    use core::arch::asm;

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

    impl Cr3 {
        #[inline]
        pub fn read() -> (PhysFrame, Cr3Flags) {
            let (frame, value) = Cr3::read_raw();
            let flags = Cr3Flags::from_bits_truncate(value.into());
            (frame, flags)
        }

        #[inline]
        pub fn read_raw() -> (PhysFrame, u16) {
            let value: u64;

            unsafe {
                asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);
            let frame = PhysFrame::containing_address(addr);
            (frame, (value & 0xFFF) as u16)
        }

        #[inline]
        pub unsafe fn write(frame: PhysFrame, flags: Cr3Flags) {
            unsafe {
                Cr3::write_raw_impl(false, frame, flags.bits() as u16);
            }
        }

        #[inline]
        unsafe fn write_raw_impl(top_bit: bool, frame: PhysFrame, val: u16) {
            let addr = frame.start_address();
            let value = ((top_bit as u64) << 63) | addr.as_u64() | val as u64;

            unsafe {
                asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
            }
        }
    }
}
