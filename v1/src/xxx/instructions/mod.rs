#![cfg(all(feature = "instructions", target_arch = "x86_64"))]

//! Special x86_64 instructions.

pub mod interrupts;
pub mod random;
pub mod segmentation;
pub mod tables;
pub mod tlb;
