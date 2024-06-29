use crate::instructions::cs_get_reg;
use crate::structures::{DescriptorTablePointer, SegmentSelector};
use crate::virt_addr::VirtAddr;
use bit_field::BitField;
use core::marker::PhantomData;

#[derive(Clone)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    divide_error: IdtEntry<HandlerFunc>,
    debug: IdtEntry<HandlerFunc>,
    non_maskable_interrupt: IdtEntry<HandlerFunc>,
    pub breakpoint: IdtEntry<HandlerFunc>,
    overflow: IdtEntry<HandlerFunc>,
    bound_range_exceeded: IdtEntry<HandlerFunc>,
    invalid_opcode: IdtEntry<HandlerFunc>,
    device_not_available: IdtEntry<HandlerFunc>,
    pub double_fault: IdtEntry<DivergingHandlerFuncWithErrCode>,
    coprocessor_segment_overrun: IdtEntry<HandlerFunc>,
    invalid_tss: IdtEntry<HandlerFuncWithErrCode>,
    segment_not_present: IdtEntry<HandlerFuncWithErrCode>,
    stack_segment_fault: IdtEntry<HandlerFuncWithErrCode>,
    general_protection_fault: IdtEntry<HandlerFuncWithErrCode>,
    page_fault: IdtEntry<HandlerFuncWithErrCode>,
    reserved_1: IdtEntry<HandlerFunc>,
    x87_floating_point: IdtEntry<HandlerFunc>,
    alignment_check: IdtEntry<HandlerFuncWithErrCode>,
    machine_check: IdtEntry<DivergingHandlerFunc>,
    simd_floating_point: IdtEntry<HandlerFunc>,
    virtualization: IdtEntry<HandlerFunc>,
    cp_protection_exception: IdtEntry<HandlerFuncWithErrCode>,
    reserved_2: [IdtEntry<HandlerFunc>; 6],
    hv_injection_exception: IdtEntry<HandlerFunc>,
    vmm_communication_exception: IdtEntry<HandlerFuncWithErrCode>,
    security_exception: IdtEntry<HandlerFuncWithErrCode>,
    reserved_3: IdtEntry<HandlerFunc>,
    interrupts: [IdtEntry<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            divide_error: IdtEntry::missing(),
            debug: IdtEntry::missing(),
            non_maskable_interrupt: IdtEntry::missing(),
            breakpoint: IdtEntry::missing(),
            overflow: IdtEntry::missing(),
            bound_range_exceeded: IdtEntry::missing(),
            invalid_opcode: IdtEntry::missing(),
            device_not_available: IdtEntry::missing(),
            double_fault: IdtEntry::missing(),
            coprocessor_segment_overrun: IdtEntry::missing(),
            invalid_tss: IdtEntry::missing(),
            segment_not_present: IdtEntry::missing(),
            stack_segment_fault: IdtEntry::missing(),
            general_protection_fault: IdtEntry::missing(),
            page_fault: IdtEntry::missing(),
            reserved_1: IdtEntry::missing(),
            x87_floating_point: IdtEntry::missing(),
            alignment_check: IdtEntry::missing(),
            machine_check: IdtEntry::missing(),
            simd_floating_point: IdtEntry::missing(),
            virtualization: IdtEntry::missing(),
            cp_protection_exception: IdtEntry::missing(),
            reserved_2: [IdtEntry::missing(); 6],
            hv_injection_exception: IdtEntry::missing(),
            vmm_communication_exception: IdtEntry::missing(),
            security_exception: IdtEntry::missing(),
            reserved_3: IdtEntry::missing(),
            interrupts: [IdtEntry::missing(); 256 - 32],
        }
    }

    pub fn load(&'static self) {
        unsafe {
            crate::instructions::lidt(&self.pointer());
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
pub struct IdtEntry<F> {
    pointer_low: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);
type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
type DivergingHandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame) -> !;
type DivergingHandlerFuncWithErrCode =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64) -> !;

macro_rules! impl_handler_func_type {
    ($f:ty) => {
        unsafe impl HandlerFuncType for $f {
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

impl<F> IdtEntry<F> {
    pub const fn missing() -> Self {
        IdtEntry {
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }
}

impl<F: HandlerFuncType> IdtEntry<F> {
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

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryOptions {
    cs: SegmentSelector,
    bits: u16,
}

impl EntryOptions {
    const fn minimal() -> Self {
        EntryOptions {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000, // Default to a 64-bit Interrupt Gate
        }
    }

    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.bits.set_bits(0..3, index + 1);
        self
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrame {
    instruction_pointer: VirtAddr,
    code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    _cpu_flags: u64,
    stack_pointer: VirtAddr,
    stack_segment: SegmentSelector,
    _reserved2: [u8; 6],
}
