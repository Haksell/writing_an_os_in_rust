use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct MxCsr: u32 {
        const INVALID_OPERATION = 1 << 0;
        const DENORMAL = 1 << 1;
        const DIVIDE_BY_ZERO = 1 << 2;
        const OVERFLOW = 1 << 3;
        const UNDERFLOW = 1 << 4;
        const PRECISION = 1 << 5;
        const DENORMALS_ARE_ZEROS = 1 << 6;
        const INVALID_OPERATION_MASK = 1 << 7;
        const DENORMAL_MASK = 1 << 8;
        const DIVIDE_BY_ZERO_MASK = 1 << 9;
        const OVERFLOW_MASK = 1 << 10;
        const UNDERFLOW_MASK = 1 << 11;
        const PRECISION_MASK = 1 << 12;
        const ROUNDING_CONTROL_NEGATIVE = 1 << 13;
        const ROUNDING_CONTROL_POSITIVE = 1 << 14;
        const ROUNDING_CONTROL_ZERO = 3 << 13;
        const FLUSH_TO_ZERO = 1 << 15;
    }
}

impl Default for MxCsr {
    #[inline]
    fn default() -> Self {
        MxCsr::INVALID_OPERATION_MASK
            | MxCsr::DENORMAL_MASK
            | MxCsr::DIVIDE_BY_ZERO_MASK
            | MxCsr::OVERFLOW_MASK
            | MxCsr::UNDERFLOW_MASK
            | MxCsr::PRECISION_MASK
    }
}
