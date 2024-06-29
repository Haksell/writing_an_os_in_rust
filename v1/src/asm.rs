use crate::xxx::{segment_selector::SegmentSelector, structures::DescriptorTablePointer};
use core::arch::asm;

#[inline]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub fn cr3_read() -> usize {
    let cr3: usize;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    cr3
}

#[inline]
pub unsafe fn cr3_write(addr: usize) {
    let value = addr as u64;
    unsafe {
        asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }
}

#[inline]
fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}

#[inline]
pub fn enable_nxe_bit() {
    const IA32_EFER: u32 = 0xC0000080;
    const NXE_BIT: u32 = 1 << 11;
    let (high, low): (u32, u32);

    unsafe {
        asm!(
            "rdmsr",
            in("ecx") IA32_EFER,
            out("eax") low, out("edx") high,
            options(nomem, nostack, preserves_flags),
        );
        asm!(
            "wrmsr",
            in("ecx") IA32_EFER,
            in("eax") (low | NXE_BIT), in("edx") high,
            options(nostack, preserves_flags),
        );
    }
}

#[inline]
pub fn enable_write_protect_bit() {
    const WRITE_PROTECT_BIT: u64 = 1 << 16;
    unsafe {
        let value: u64;
        asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
        asm!("mov cr0, {}", in(reg) (value | WRITE_PROTECT_BIT), options(nostack, preserves_flags));
    }
}

#[inline]
pub fn tlb_flush(addr: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

#[inline]
pub fn tlb_flush_all() {
    let value: u64;

    unsafe {
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        asm!("mov cr3, {}", in(reg) (value & 0x_000f_ffff_ffff_ffff), options(nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    unsafe {
        asm!("ltr {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn cs_get_reg() -> SegmentSelector {
    let segment: u16;
    unsafe {
        asm!(concat!("mov {0:x}, cs"), out(reg) segment, options(nomem, nostack, preserves_flags));
    }
    SegmentSelector(segment)
}

#[inline]
pub unsafe fn cs_set_reg(sel: SegmentSelector) {
    unsafe {
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(sel.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
    }
}
