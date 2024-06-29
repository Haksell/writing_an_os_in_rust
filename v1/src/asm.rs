use crate::xxx::registers::model_specific::Msr;
use crate::xxx::registers::segmentation::SegmentSelector;
use crate::xxx::structures::DescriptorTablePointer;
use core::arch::asm;

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
    const NXE_BIT: u64 = 1 << 11;

    let mut ia32_efer = Msr::new(IA32_EFER);
    unsafe {
        ia32_efer.write(ia32_efer.read() | NXE_BIT);
    }
}

#[inline]
pub fn enable_write_protect_bit() {
    let value: u64;

    unsafe {
        asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
        asm!("mov cr0, {}", in(reg) (value | 1 << 16), options(nostack, preserves_flags));
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
    }

    let value = value & 0x_000f_ffff_ffff_ffff;

    unsafe {
        asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    unsafe {
        asm!("ltr {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
    }
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
