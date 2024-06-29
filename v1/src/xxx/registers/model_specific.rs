use core::arch::asm;

#[derive(Debug)]
pub struct Msr(u32);

impl Msr {
    #[inline]
    pub const fn new(reg: u32) -> Msr {
        Msr(reg)
    }
}

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
