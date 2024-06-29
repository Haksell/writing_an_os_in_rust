use core::fmt;

/// Structure of a PCID. A PCID has to be <= 4096 for x86_64.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pcid(u16);

/// A passed `u16` was not a valid PCID.
///
/// A PCID has to be <= 4096 for x86_64.
#[derive(Debug)]
pub struct PcidTooBig(u16);

impl fmt::Display for PcidTooBig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PCID should be < 4096, got {}", self.0)
    }
}

/// An error returned when trying to use an invalid ASID.
#[derive(Debug)]
pub struct AsidOutOfRangeError {
    /// The requested ASID.
    pub asid: u16,
    /// The number of valid ASIDS.
    pub nasid: u32,
}

impl fmt::Display for AsidOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} is out of the range of available ASIDS ({})",
            self.asid, self.nasid
        )
    }
}
