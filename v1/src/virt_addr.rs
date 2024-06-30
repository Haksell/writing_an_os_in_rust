#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> VirtAddr {
        let truncated = ((addr << 16) as i64 >> 16) as u64;
        if truncated == addr {
            VirtAddr(truncated)
        } else {
            panic!("virtual address must be sign extended in bits 48 to 64");
        }
    }

    pub const fn zero() -> VirtAddr {
        VirtAddr(0)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}
