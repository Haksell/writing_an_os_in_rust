use crate::asm::cs_get_reg;
use crate::structures::{DescriptorTablePointer, SegmentSelector};
use crate::virt_addr::VirtAddr;
use bit_field::BitField;
use bitflags::bitflags;
use core::marker::PhantomData;

#[derive(Clone)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    divide_error: Entry<HandlerFunc>,
    debug: Entry<HandlerFunc>,
    non_maskable_interrupt: Entry<HandlerFunc>,
    pub breakpoint: Entry<HandlerFunc>,
    overflow: Entry<HandlerFunc>,
    bound_range_exceeded: Entry<HandlerFunc>,
    invalid_opcode: Entry<HandlerFunc>,
    device_not_available: Entry<HandlerFunc>,
    pub double_fault: Entry<DivergingHandlerFuncWithErrCode>,
    coprocessor_segment_overrun: Entry<HandlerFunc>,
    invalid_tss: Entry<HandlerFuncWithErrCode>,
    segment_not_present: Entry<HandlerFuncWithErrCode>,
    stack_segment_fault: Entry<HandlerFuncWithErrCode>,
    general_protection_fault: Entry<HandlerFuncWithErrCode>,
    page_fault: Entry<HandlerFuncWithErrCode>,
    reserved_1: Entry<HandlerFunc>,
    x87_floating_point: Entry<HandlerFunc>,
    alignment_check: Entry<HandlerFuncWithErrCode>,
    machine_check: Entry<DivergingHandlerFunc>,
    simd_floating_point: Entry<HandlerFunc>,
    virtualization: Entry<HandlerFunc>,
    cp_protection_exception: Entry<HandlerFuncWithErrCode>,
    reserved_2: [Entry<HandlerFunc>; 6],
    hv_injection_exception: Entry<HandlerFunc>,
    vmm_communication_exception: Entry<HandlerFuncWithErrCode>,
    security_exception: Entry<HandlerFuncWithErrCode>,
    reserved_3: Entry<HandlerFunc>,
    interrupts: [Entry<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
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
        unsafe {
            crate::asm::lidt(&self.pointer());
        }
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (core::mem::size_of::<Self>() - 1) as u16,
        }
    }
}

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

pub type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
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
}

impl<F: HandlerFuncType> Entry<F> {
    pub fn set_handler_fn(&mut self, handler: F) -> &mut EntryOptions {
        const PRESENT_BIT: usize = 15;
        let addr = handler.to_virt_addr().as_u64();
        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;
        self.options = EntryOptions::minimal();
        unsafe { self.options.cs = cs_get_reg() };
        self.options.bits.set_bit(PRESENT_BIT, true);
        &mut self.options
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
impl_handler_func_type!(DivergingHandlerFunc);
impl_handler_func_type!(DivergingHandlerFuncWithErrCode);

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryOptions {
    cs: SegmentSelector,
    bits: u16,
}

impl EntryOptions {
    #[inline]
    const fn minimal() -> Self {
        EntryOptions {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000, // Default to a 64-bit Interrupt Gate
        }
    }

    #[inline]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.bits.set_bits(0..3, index + 1);
        self
    }
}

#[repr(transparent)]
pub struct InterruptStackFrame(InterruptStackFrameValue);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    instruction_pointer: VirtAddr,
    code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    _cpu_flags: u64,
    stack_pointer: VirtAddr,
    stack_segment: SegmentSelector,
    _reserved2: [u8; 6],
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SelectorErrorCode {
    flags: u64,
}
