//! This crate provides x86_64 specific functions and data structures,
//! and access to various system registers.

#![deny(missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]

pub use addr::{PhysAddr, VirtAddr};

pub mod addr;
pub mod instructions;
pub mod registers;
pub mod structures;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl PrivilegeLevel {
    #[inline]
    pub const fn from_u16(value: u16) -> PrivilegeLevel {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
}
