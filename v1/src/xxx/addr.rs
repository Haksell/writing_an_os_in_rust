#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    #[inline]
    pub const fn new(addr: u64) -> VirtAddr {
        // TODO: fix addresses that should be sign-extended with 1
        match Self::try_new(addr) {
            Some(v) => v,
            None => panic!("virtual address must be sign extended in bits 48 to 64"),
        }
    }

    #[inline]
    pub const fn try_new(addr: u64) -> Option<VirtAddr> {
        let v = Self::new_truncate(addr);
        if v.0 == addr {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub const fn new_truncate(addr: u64) -> VirtAddr {
        VirtAddr(((addr << 16) as i64 >> 16) as u64)
    }

    #[inline]
    pub const fn zero() -> VirtAddr {
        VirtAddr(0)
    }

    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}
