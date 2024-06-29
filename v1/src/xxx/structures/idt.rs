use crate::xxx::registers::rflags::RFlags;
use crate::xxx::{PrivilegeLevel, VirtAddr};
use bit_field::BitField;
use bitflags::bitflags;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Bound::{Excluded, Included, Unbounded};
use core::ops::{
    Bound, Deref, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive,
    RangeTo, RangeToInclusive,
};

use super::gdt::SegmentSelector;

#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error: Entry<HandlerFunc>,

    pub debug: Entry<HandlerFunc>,

    pub non_maskable_interrupt: Entry<HandlerFunc>,

    pub breakpoint: Entry<HandlerFunc>,

    pub overflow: Entry<HandlerFunc>,

    pub bound_range_exceeded: Entry<HandlerFunc>,

    pub invalid_opcode: Entry<HandlerFunc>,

    pub device_not_available: Entry<HandlerFunc>,

    pub double_fault: Entry<DivergingHandlerFuncWithErrCode>,

    coprocessor_segment_overrun: Entry<HandlerFunc>,

    pub invalid_tss: Entry<HandlerFuncWithErrCode>,

    pub segment_not_present: Entry<HandlerFuncWithErrCode>,

    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,

    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,

    pub page_fault: Entry<PageFaultHandlerFunc>,

    reserved_1: Entry<HandlerFunc>,

    pub x87_floating_point: Entry<HandlerFunc>,

    pub alignment_check: Entry<HandlerFuncWithErrCode>,

    pub machine_check: Entry<DivergingHandlerFunc>,

    pub simd_floating_point: Entry<HandlerFunc>,

    pub virtualization: Entry<HandlerFunc>,

    pub cp_protection_exception: Entry<HandlerFuncWithErrCode>,

    reserved_2: [Entry<HandlerFunc>; 6],

    pub hv_injection_exception: Entry<HandlerFunc>,

    pub vmm_communication_exception: Entry<HandlerFuncWithErrCode>,

    pub security_exception: Entry<HandlerFuncWithErrCode>,

    reserved_3: Entry<HandlerFunc>,

    interrupts: [Entry<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    #[inline]
    pub fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            divide_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            coprocessor_segment_overrun: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            reserved_1: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            cp_protection_exception: Entry::missing(),
            reserved_2: [Entry::missing(); 6],
            hv_injection_exception: Entry::missing(),
            vmm_communication_exception: Entry::missing(),
            security_exception: Entry::missing(),
            reserved_3: Entry::missing(),
            interrupts: [Entry::missing(); 256 - 32],
        }
    }

    #[inline]
    pub fn load(&'static self) {
        unsafe { self.load_unsafe() }
    }

    #[inline]
    pub unsafe fn load_unsafe(&self) {
        unsafe {
            crate::xxx::instructions::tables::lidt(&self.pointer());
        }
    }

    fn pointer(&self) -> crate::xxx::structures::DescriptorTablePointer {
        use core::mem::size_of;
        crate::xxx::structures::DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        }
    }

    fn condition_slice_bounds(&self, bounds: impl RangeBounds<u8>) -> (usize, usize) {
        let lower_idx = match bounds.start_bound() {
            Included(start) => usize::from(*start),
            Excluded(start) => usize::from(*start) + 1,
            Unbounded => 0,
        };
        let upper_idx = match bounds.end_bound() {
            Included(end) => usize::from(*end) + 1,
            Excluded(end) => usize::from(*end),
            Unbounded => 256,
        };

        if lower_idx < 32 {
            panic!("Cannot return slice from traps, faults, and exception handlers");
        }
        (lower_idx, upper_idx)
    }

    #[inline]
    pub fn slice(&self, bounds: impl RangeBounds<u8>) -> &[Entry<HandlerFunc>] {
        let (lower_idx, upper_idx) = self.condition_slice_bounds(bounds);
        &self.interrupts[(lower_idx - 32)..(upper_idx - 32)]
    }

    #[inline]
    pub fn slice_mut(&mut self, bounds: impl RangeBounds<u8>) -> &mut [Entry<HandlerFunc>] {
        let (lower_idx, upper_idx) = self.condition_slice_bounds(bounds);
        &mut self.interrupts[(lower_idx - 32)..(upper_idx - 32)]
    }
}

impl Default for InterruptDescriptorTable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u8> for InterruptDescriptorTable {
    type Output = Entry<HandlerFunc>;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.divide_error,
            1 => &self.debug,
            2 => &self.non_maskable_interrupt,
            3 => &self.breakpoint,
            4 => &self.overflow,
            5 => &self.bound_range_exceeded,
            6 => &self.invalid_opcode,
            7 => &self.device_not_available,
            9 => &self.coprocessor_segment_overrun,
            16 => &self.x87_floating_point,
            19 => &self.simd_floating_point,
            20 => &self.virtualization,
            28 => &self.hv_injection_exception,
            i @ 32..=255 => &self.interrupts[usize::from(i) - 32],
            i @ 15 | i @ 31 | i @ 22..=27 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 21 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is an diverging exception (must not return)", i),
        }
    }
}

impl IndexMut<u8> for InterruptDescriptorTable {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.divide_error,
            1 => &mut self.debug,
            2 => &mut self.non_maskable_interrupt,
            3 => &mut self.breakpoint,
            4 => &mut self.overflow,
            5 => &mut self.bound_range_exceeded,
            6 => &mut self.invalid_opcode,
            7 => &mut self.device_not_available,
            9 => &mut self.coprocessor_segment_overrun,
            16 => &mut self.x87_floating_point,
            19 => &mut self.simd_floating_point,
            20 => &mut self.virtualization,
            28 => &mut self.hv_injection_exception,
            i @ 32..=255 => &mut self.interrupts[usize::from(i) - 32],
            i @ 15 | i @ 31 | i @ 22..=27 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 21 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is an diverging exception (must not return)", i),
        }
    }
}

macro_rules! impl_index_for_idt {
    ($ty:ty) => {
        impl Index<$ty> for InterruptDescriptorTable {
            type Output = [Entry<HandlerFunc>];

            #[inline]
            fn index(&self, index: $ty) -> &Self::Output {
                self.slice(index)
            }
        }

        impl IndexMut<$ty> for InterruptDescriptorTable {
            #[inline]
            fn index_mut(&mut self, index: $ty) -> &mut Self::Output {
                self.slice_mut(index)
            }
        }
    };
}

// this list was stolen from the list of implementors in https://doc.rust-lang.org/core/ops/trait.RangeBounds.html
impl_index_for_idt!((Bound<&u8>, Bound<&u8>));
impl_index_for_idt!((Bound<u8>, Bound<u8>));
impl_index_for_idt!(Range<&u8>);
impl_index_for_idt!(Range<u8>);
impl_index_for_idt!(RangeFrom<&u8>);
impl_index_for_idt!(RangeFrom<u8>);
impl_index_for_idt!(RangeInclusive<&u8>);
impl_index_for_idt!(RangeInclusive<u8>);
impl_index_for_idt!(RangeTo<u8>);
impl_index_for_idt!(RangeTo<&u8>);
impl_index_for_idt!(RangeToInclusive<&u8>);
impl_index_for_idt!(RangeToInclusive<u8>);
impl_index_for_idt!(RangeFull);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Entry<F> {
    pointer_low: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

impl<T> fmt::Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("handler_addr", &format_args!("{:#x}", self.handler_addr()))
            .field("options", &self.options)
            .finish()
    }
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer_low == other.pointer_low
            && self.options == other.options
            && self.pointer_middle == other.pointer_middle
            && self.pointer_high == other.pointer_high
            && self.reserved == other.reserved
    }
}

pub type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
pub type PageFaultHandlerFunc =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErrorCode);
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame) -> !;
pub type DivergingHandlerFuncWithErrCode =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64) -> !;

impl<F> Entry<F> {
    #[inline]
    pub const fn missing() -> Self {
        Entry {
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn set_handler_addr(&mut self, addr: VirtAddr) -> &mut EntryOptions {
        use crate::xxx::instructions::segmentation::{Segment, CS};

        let addr = addr.as_u64();
        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.options = EntryOptions::minimal();
        // SAFETY: The current CS is a valid, long-mode code segment.
        unsafe { self.options.set_code_selector(CS::get_reg()) };
        self.options.set_present(true);
        &mut self.options
    }

    #[inline]
    pub fn handler_addr(&self) -> VirtAddr {
        let addr = self.pointer_low as u64
            | (self.pointer_middle as u64) << 16
            | (self.pointer_high as u64) << 32;
        // addr is a valid VirtAddr, as the pointer members are either all zero,
        // or have been set by set_handler_addr (which takes a VirtAddr).
        VirtAddr::new_truncate(addr)
    }
}

impl<F: HandlerFuncType> Entry<F> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: F) -> &mut EntryOptions {
        unsafe { self.set_handler_addr(handler.to_virt_addr()) }
    }
}

pub unsafe trait HandlerFuncType {
    fn to_virt_addr(self) -> VirtAddr;
}

macro_rules! impl_handler_func_type {
    ($f:ty) => {
        unsafe impl HandlerFuncType for $f {
            #[inline]
            fn to_virt_addr(self) -> VirtAddr {
                VirtAddr::new(self as u64)
            }
        }
    };
}

impl_handler_func_type!(HandlerFunc);
impl_handler_func_type!(HandlerFuncWithErrCode);
impl_handler_func_type!(PageFaultHandlerFunc);
impl_handler_func_type!(DivergingHandlerFunc);
impl_handler_func_type!(DivergingHandlerFuncWithErrCode);

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryOptions {
    cs: SegmentSelector,
    bits: u16,
}

impl fmt::Debug for EntryOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EntryOptions")
            .field("code_selector", &self.cs)
            .field("stack_index", &self.stack_index())
            .field("type", &format_args!("{:#04b}", self.bits.get_bits(8..12)))
            .field("privilege_level", &self.privilege_level())
            .field("present", &self.present())
            .finish()
    }
}

impl EntryOptions {
    #[inline]
    const fn minimal() -> Self {
        EntryOptions {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000, // Default to a 64-bit Interrupt Gate
        }
    }

    pub unsafe fn set_code_selector(&mut self, cs: SegmentSelector) -> &mut Self {
        self.cs = cs;
        self
    }

    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.bits.set_bit(15, present);
        self
    }

    fn present(&self) -> bool {
        self.bits.get_bit(15)
    }

    fn privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.bits.get_bits(13..15))
    }

    #[inline]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.bits.set_bits(0..3, index + 1);
        self
    }

    fn stack_index(&self) -> u16 {
        self.bits.get_bits(0..3) - 1
    }
}

#[repr(transparent)]
pub struct InterruptStackFrame(InterruptStackFrameValue);

impl Deref for InterruptStackFrame {
    type Target = InterruptStackFrameValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for InterruptStackFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    pub instruction_pointer: VirtAddr,
    pub code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    pub cpu_flags: RFlags,
    pub stack_pointer: VirtAddr,
    pub stack_segment: SegmentSelector,
    _reserved2: [u8; 6],
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &self.cpu_flags);
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1;

        const CAUSED_BY_WRITE = 1 << 1;

        const USER_MODE = 1 << 2;

        const MALFORMED_TABLE = 1 << 3;

        const INSTRUCTION_FETCH = 1 << 4;

        const PROTECTION_KEY = 1 << 5;

        const SHADOW_STACK = 1 << 6;

        const SGX = 1 << 15;

        const RMP = 1 << 31;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SelectorErrorCode {
    flags: u64,
}

impl SelectorErrorCode {
    pub fn external(&self) -> bool {
        self.flags.get_bit(0)
    }

    pub fn descriptor_table(&self) -> DescriptorTable {
        match self.flags.get_bits(1..3) {
            0b00 => DescriptorTable::Gdt,
            0b01 => DescriptorTable::Idt,
            0b10 => DescriptorTable::Ldt,
            0b11 => DescriptorTable::Idt,
            _ => unreachable!(),
        }
    }

    pub fn index(&self) -> u64 {
        self.flags.get_bits(3..16)
    }
}

impl fmt::Debug for SelectorErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("Selector Error");
        s.field("external", &self.external());
        s.field("descriptor table", &self.descriptor_table());
        s.field("index", &self.index());
        s.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorTable {
    Gdt,
    Idt,
    Ldt,
}
