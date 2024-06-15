use core::arch::asm;
use x86_64::{
    registers::{
        control::{Cr0, Cr0Flags},
        model_specific::Msr,
    },
    structures::DescriptorTablePointer,
};

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

pub fn enable_nxe_bit() {
    const IA32_EFER: u32 = 0xC0000080;
    const NXE_BIT: u64 = 1 << 11;

    let mut ia32_efer = Msr::new(IA32_EFER);
    unsafe {
        ia32_efer.write(ia32_efer.read() | NXE_BIT);
    }
}

pub fn enable_write_protect_bit() {
    unsafe {
        Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT);
    }
}
