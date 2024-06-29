//! Functions to read and write model specific registers.

use bitflags::bitflags;

#[derive(Debug)]
pub struct Msr(u32);

impl Msr {
    #[inline]
    pub const fn new(reg: u32) -> Msr {
        Msr(reg)
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct EferFlags: u64 {
        const SYSTEM_CALL_EXTENSIONS = 1;
        const LONG_MODE_ENABLE = 1 << 8;
        const LONG_MODE_ACTIVE = 1 << 10;
        const NO_EXECUTE_ENABLE = 1 << 11;
        const SECURE_VIRTUAL_MACHINE_ENABLE = 1 << 12;
        const LONG_MODE_SEGMENT_LIMIT_ENABLE = 1 << 13;
        const FAST_FXSAVE_FXRSTOR = 1 << 14;
        const TRANSLATION_CACHE_EXTENSION = 1 << 15;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct CetFlags: u64 {
        const SS_ENABLE = 1 << 0;
        const SS_WRITE_ENABLE = 1 << 1;
        const IBT_ENABLE = 1 << 2;
        const IBT_LEGACY_ENABLE = 1 << 3;
        const IBT_NO_TRACK_ENABLE = 1 << 4;
        const IBT_LEGACY_SUPPRESS_ENABLE = 1 << 5;
        const IBT_SUPPRESS_ENABLE = 1 << 10;
        const IBT_TRACKED = 1 << 11;
    }
}

mod x86_64 {
    use super::*;
    use core::arch::asm;

    impl Msr {
        #[inline]
        pub unsafe fn read(&self) -> u64 {
            let (high, low): (u32, u32);
            unsafe {
                asm!(
                    "rdmsr",
                    in("ecx") self.0,
                    out("eax") low, out("edx") high,
                    options(nomem, nostack, preserves_flags),
                );
            }
            ((high as u64) << 32) | (low as u64)
        }

        #[inline]
        pub unsafe fn write(&mut self, value: u64) {
            let low = value as u32;
            let high = (value >> 32) as u32;

            unsafe {
                asm!(
                    "wrmsr",
                    in("ecx") self.0,
                    in("eax") low, in("edx") high,
                    options(nostack, preserves_flags),
                );
            }
        }
    }
}
