//! Functions to read and write model specific registers.

use bitflags::bitflags;

#[derive(Debug)]
pub struct Msr(u32);

impl Msr {
    /// Create an instance from a register.
    #[inline]
    pub const fn new(reg: u32) -> Msr {
        Msr(reg)
    }
}

/// [FS].Base Model Specific Register.
#[derive(Debug)]
pub struct FsBase;

#[derive(Debug)]
pub struct GsBase;

impl FsBase {
    /// The underlying model specific register.
    pub const MSR: Msr = Msr(0xC000_0100);
}

impl GsBase {
    /// The underlying model specific register.
    pub const MSR: Msr = Msr(0xC000_0101);
}

bitflags! {
    /// Flags of the Extended Feature Enable Register.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct EferFlags: u64 {
        /// Enables the `syscall` and `sysret` instructions.
        const SYSTEM_CALL_EXTENSIONS = 1;
        /// Activates long mode, requires activating paging.
        const LONG_MODE_ENABLE = 1 << 8;
        /// Indicates that long mode is active.
        const LONG_MODE_ACTIVE = 1 << 10;
        /// Enables the no-execute page-protection feature.
        const NO_EXECUTE_ENABLE = 1 << 11;
        /// Enables SVM extensions.
        const SECURE_VIRTUAL_MACHINE_ENABLE = 1 << 12;
        /// Enable certain limit checks in 64-bit mode.
        const LONG_MODE_SEGMENT_LIMIT_ENABLE = 1 << 13;
        /// Enable the `fxsave` and `fxrstor` instructions to execute faster in 64-bit mode.
        const FAST_FXSAVE_FXRSTOR = 1 << 14;
        /// Changes how the `invlpg` instruction operates on TLB entries of upper-level entries.
        const TRANSLATION_CACHE_EXTENSION = 1 << 15;
    }
}

bitflags! {
    /// Flags stored in IA32_U_CET and IA32_S_CET (Table-2-2 in Intel SDM Volume
    /// 4). The Intel SDM-equivalent names are described in parentheses.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct CetFlags: u64 {
        /// Enable shadow stack (SH_STK_EN)
        const SS_ENABLE = 1 << 0;
        /// Enable WRSS{D,Q}W instructions (WR_SHTK_EN)
        const SS_WRITE_ENABLE = 1 << 1;
        /// Enable indirect branch tracking (ENDBR_EN)
        const IBT_ENABLE = 1 << 2;
        /// Enable legacy treatment for indirect branch tracking (LEG_IW_EN)
        const IBT_LEGACY_ENABLE = 1 << 3;
        /// Enable no-track opcode prefix for indirect branch tracking (NO_TRACK_EN)
        const IBT_NO_TRACK_ENABLE = 1 << 4;
        /// Disable suppression of CET on legacy compatibility (SUPPRESS_DIS)
        const IBT_LEGACY_SUPPRESS_ENABLE = 1 << 5;
        /// Enable suppression of indirect branch tracking (SUPPRESS)
        const IBT_SUPPRESS_ENABLE = 1 << 10;
        /// Is IBT waiting for a branch to return? (read-only, TRACKER)
        const IBT_TRACKED = 1 << 11;
    }
}

mod x86_64 {
    use super::*;
    use crate::xxx::addr::VirtAddr;
    use core::arch::asm;
    use core::fmt;

    impl Msr {
        /// Read 64 bits msr register.
        ///
        /// ## Safety
        ///
        /// The caller must ensure that this read operation has no unsafe side
        /// effects.
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

        /// Write 64 bits to msr register.
        ///
        /// ## Safety
        ///
        /// The caller must ensure that this write operation has no unsafe side
        /// effects.
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

    #[derive(Debug)]
    pub enum InvalidStarSegmentSelectors {
        SysretOffset,
        SyscallOffset,
        SysretPrivilegeLevel,
        SyscallPrivilegeLevel,
    }

    impl fmt::Display for InvalidStarSegmentSelectors {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::SysretOffset => write!(f, "Sysret CS and SS are not offset by 8."),
                Self::SyscallOffset => write!(f, "Syscall CS and SS are not offset by 8."),
                Self::SysretPrivilegeLevel => {
                    write!(f, "Sysret's segment must be a Ring3 segment.")
                }
                Self::SyscallPrivilegeLevel => {
                    write!(f, "Syscall's segment must be a Ring0 segment.")
                }
            }
        }
    }
}
