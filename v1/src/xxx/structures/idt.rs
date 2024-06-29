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

/// An Interrupt Descriptor Table with 256 entries.
///
/// The first 32 entries are used for CPU exceptions. These entries can be either accessed through
/// fields on this struct or through an index operation, e.g. `idt[0]` returns the
/// first entry, the entry for the `divide_error` exception. Note that the index access is
/// not possible for entries for which an error code is pushed.
///
/// The remaining entries are used for interrupts. They can be accessed through index
/// operations on the idt, e.g. `idt[32]` returns the first interrupt entry, which is the 32nd IDT
/// entry).
///
///
/// The field descriptions are taken from the
/// [AMD64 manual volume 2](https://support.amd.com/TechDocs/24593.pdf)
/// (with slight modifications).
#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    /// A divide error (`#DE`) occurs when the denominator of a DIV instruction or
    /// an IDIV instruction is 0. A `#DE` also occurs if the result is too large to be
    /// represented in the destination.
    ///
    /// The saved instruction pointer points to the instruction that caused the `#DE`.
    ///
    /// The vector number of the `#DE` exception is 0.
    pub divide_error: Entry<HandlerFunc>,

    /// When the debug-exception mechanism is enabled, a `#DB` exception can occur under any
    /// of the following circumstances:
    ///
    /// <details>
    ///
    /// - Instruction execution.
    /// - Instruction single stepping.
    /// - Data read.
    /// - Data write.
    /// - I/O read.
    /// - I/O write.
    /// - Task switch.
    /// - Debug-register access, or general detect fault (debug register access when DR7.GD=1).
    /// - Executing the INT1 instruction (opcode 0F1h).
    ///
    /// </details>
    ///
    /// `#DB` conditions are enabled and disabled using the debug-control register, `DR7`
    /// and `RFLAGS.TF`.
    ///
    /// In the following cases, the saved instruction pointer points to the instruction that
    /// caused the `#DB`:
    ///
    /// - Instruction execution.
    /// - Invalid debug-register access, or general detect.
    ///
    /// In all other cases, the instruction that caused the `#DB` is completed, and the saved
    /// instruction pointer points to the instruction after the one that caused the `#DB`.
    ///
    /// The vector number of the `#DB` exception is 1.
    pub debug: Entry<HandlerFunc>,

    /// An non maskable interrupt exception (NMI) occurs as a result of system logic
    /// signaling a non-maskable interrupt to the processor.
    ///
    /// The processor recognizes an NMI at an instruction boundary.
    /// The saved instruction pointer points to the instruction immediately following the
    /// boundary where the NMI was recognized.
    ///
    /// The vector number of the NMI exception is 2.
    pub non_maskable_interrupt: Entry<HandlerFunc>,

    /// A breakpoint (`#BP`) exception occurs when an `INT3` instruction is executed. The
    /// `INT3` is normally used by debug software to set instruction breakpoints by replacing
    ///
    /// The saved instruction pointer points to the byte after the `INT3` instruction.
    ///
    /// The vector number of the `#BP` exception is 3.
    pub breakpoint: Entry<HandlerFunc>,

    /// An overflow exception (`#OF`) occurs as a result of executing an `INTO` instruction
    /// while the overflow bit in `RFLAGS` is set to 1.
    ///
    /// The saved instruction pointer points to the instruction following the `INTO`
    /// instruction that caused the `#OF`.
    ///
    /// The vector number of the `#OF` exception is 4.
    pub overflow: Entry<HandlerFunc>,

    /// A bound-range exception (`#BR`) exception can occur as a result of executing
    /// the `BOUND` instruction. The `BOUND` instruction compares an array index (first
    /// operand) with the lower bounds and upper bounds of an array (second operand).
    /// If the array index is not within the array boundary, the `#BR` occurs.
    ///
    /// The saved instruction pointer points to the `BOUND` instruction that caused the `#BR`.
    ///
    /// The vector number of the `#BR` exception is 5.
    pub bound_range_exceeded: Entry<HandlerFunc>,

    /// An invalid opcode exception (`#UD`) occurs when an attempt is made to execute an
    /// invalid or undefined opcode. The validity of an opcode often depends on the
    /// processor operating mode.
    ///
    /// <details><summary>A `#UD` occurs under the following conditions:</summary>
    ///
    /// - Execution of any reserved or undefined opcode in any mode.
    /// - Execution of the `UD2` instruction.
    /// - Use of the `LOCK` prefix on an instruction that cannot be locked.
    /// - Use of the `LOCK` prefix on a lockable instruction with a non-memory target location.
    /// - Execution of an instruction with an invalid-operand type.
    /// - Execution of the `SYSENTER` or `SYSEXIT` instructions in long mode.
    /// - Execution of any of the following instructions in 64-bit mode: `AAA`, `AAD`,
    ///   `AAM`, `AAS`, `BOUND`, `CALL` (opcode 9A), `DAA`, `DAS`, `DEC`, `INC`, `INTO`,
    ///   `JMP` (opcode EA), `LDS`, `LES`, `POP` (`DS`, `ES`, `SS`), `POPA`, `PUSH` (`CS`,
    ///   `DS`, `ES`, `SS`), `PUSHA`, `SALC`.
    /// - Execution of the `ARPL`, `LAR`, `LLDT`, `LSL`, `LTR`, `SLDT`, `STR`, `VERR`, or
    ///   `VERW` instructions when protected mode is not enabled, or when virtual-8086 mode
    ///   is enabled.
    /// - Execution of any legacy SSE instruction when `CR4.OSFXSR` is cleared to 0.
    /// - Execution of any SSE instruction (uses `YMM`/`XMM` registers), or 64-bit media
    ///   instruction (uses `MMXTM` registers) when `CR0.EM` = 1.
    /// - Execution of any SSE floating-point instruction (uses `YMM`/`XMM` registers) that
    ///   causes a numeric exception when `CR4.OSXMMEXCPT` = 0.
    /// - Use of the `DR4` or `DR5` debug registers when `CR4.DE` = 1.
    /// - Execution of `RSM` when not in `SMM` mode.
    ///
    /// </details>
    ///
    /// The saved instruction pointer points to the instruction that caused the `#UD`.
    ///
    /// The vector number of the `#UD` exception is 6.
    pub invalid_opcode: Entry<HandlerFunc>,

    /// A device not available exception (`#NM`) occurs under any of the following conditions:
    ///
    /// <details>
    ///
    /// - An `FWAIT`/`WAIT` instruction is executed when `CR0.MP=1` and `CR0.TS=1`.
    /// - Any x87 instruction other than `FWAIT` is executed when `CR0.EM=1`.
    /// - Any x87 instruction is executed when `CR0.TS=1`. The `CR0.MP` bit controls whether the
    ///   `FWAIT`/`WAIT` instruction causes an `#NM` exception when `TS=1`.
    /// - Any 128-bit or 64-bit media instruction when `CR0.TS=1`.
    ///
    /// </details>
    ///
    /// The saved instruction pointer points to the instruction that caused the `#NM`.
    ///
    /// The vector number of the `#NM` exception is 7.
    pub device_not_available: Entry<HandlerFunc>,

    /// A double fault (`#DF`) exception can occur when a second exception occurs during
    /// the handling of a prior (first) exception or interrupt handler.
    ///
    /// <details>
    ///
    /// Usually, the first and second exceptions can be handled sequentially without
    /// resulting in a `#DF`. In this case, the first exception is considered _benign_, as
    /// it does not harm the ability of the processor to handle the second exception. In some
    /// cases, however, the first exception adversely affects the ability of the processor to
    /// handle the second exception. These exceptions contribute to the occurrence of a `#DF`,
    /// and are called _contributory exceptions_. The following exceptions are contributory:
    ///
    /// - Invalid-TSS Exception
    /// - Segment-Not-Present Exception
    /// - Stack Exception
    /// - General-Protection Exception
    ///
    /// A double-fault exception occurs in the following cases:
    ///
    /// - If a contributory exception is followed by another contributory exception.
    /// - If a divide-by-zero exception is followed by a contributory exception.
    /// - If a page  fault is followed by another page fault or a contributory exception.
    ///
    /// If a third interrupting event occurs while transferring control to the `#DF` handler,
    /// the processor shuts down.
    ///
    /// </details>
    ///
    /// The returned error code is always zero. The saved instruction pointer is undefined,
    /// and the program cannot be restarted.
    ///
    /// The vector number of the `#DF` exception is 8.
    pub double_fault: Entry<DivergingHandlerFuncWithErrCode>,

    /// This interrupt vector is reserved. It is for a discontinued exception originally used
    /// by processors that supported external x87-instruction coprocessors. On those processors,
    /// the exception condition is caused by an invalid-segment or invalid-page access on an
    /// x87-instruction coprocessor-instruction operand. On current processors, this condition
    /// causes a general-protection exception to occur.
    coprocessor_segment_overrun: Entry<HandlerFunc>,

    /// An invalid TSS exception (`#TS`) occurs only as a result of a control transfer through
    /// a gate descriptor that results in an invalid stack-segment reference using an `SS`
    /// selector in the TSS.
    ///
    /// The returned error code is the `SS` segment selector. The saved instruction pointer
    /// points to the control-transfer instruction that caused the `#TS`.
    ///
    /// The vector number of the `#TS` exception is 10.
    pub invalid_tss: Entry<HandlerFuncWithErrCode>,

    /// An segment-not-present exception (`#NP`) occurs when an attempt is made to load a
    /// segment or gate with a clear present bit.
    ///
    /// The returned error code is the segment-selector index of the segment descriptor
    /// causing the `#NP` exception. The saved instruction pointer points to the instruction
    /// that loaded the segment selector resulting in the `#NP`.
    ///
    /// The vector number of the `#NP` exception is 11.
    pub segment_not_present: Entry<HandlerFuncWithErrCode>,

    /// An stack segment exception (`#SS`) can occur in the following situations:
    ///
    /// - Implied stack references in which the stack address is not in canonical
    ///   form. Implied stack references include all push and pop instructions, and any
    ///   instruction using `RSP` or `RBP` as a base register.
    /// - Attempting to load a stack-segment selector that references a segment descriptor
    ///   containing a clear present bit.
    /// - Any stack access that fails the stack-limit check.
    ///
    /// The returned error code depends on the cause of the `#SS`. If the cause is a cleared
    /// present bit, the error code is the corresponding segment selector. Otherwise, the
    /// error code is zero. The saved instruction pointer points to the instruction that
    /// caused the `#SS`.
    ///
    /// The vector number of the `#NP` exception is 12.
    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,

    /// A general protection fault (`#GP`) can occur in various situations. Common causes include:
    ///
    /// - Executing a privileged instruction while `CPL > 0`.
    /// - Writing a 1 into any register field that is reserved, must be zero (MBZ).
    /// - Attempting to execute an SSE instruction specifying an unaligned memory operand.
    /// - Loading a non-canonical base address into the `GDTR` or `IDTR`.
    /// - Using WRMSR to write a read-only MSR.
    /// - Any long-mode consistency-check violation.
    ///
    /// The returned error code is a segment selector, if the cause of the `#GP` is
    /// segment-related, and zero otherwise. The saved instruction pointer points to
    /// the instruction that caused the `#GP`.
    ///
    /// The vector number of the `#GP` exception is 13.
    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,

    /// A page fault (`#PF`) can occur during a memory access in any of the following situations:
    ///
    /// - A page-translation-table entry or physical page involved in translating the memory
    ///   access is not present in physical memory. This is indicated by a cleared present
    ///   bit in the translation-table entry.
    /// - An attempt is made by the processor to load the instruction TLB with a translation
    ///   for a non-executable page.
    /// - The memory access fails the paging-protection checks (user/supervisor, read/write,
    ///   or both).
    /// - A reserved bit in one of the page-translation-table entries is set to 1. A `#PF`
    ///   occurs for this reason only when `CR4.PSE=1` or `CR4.PAE=1`.
    ///
    /// The virtual (linear) address that caused the `#PF` is stored in the `CR2` register.
    /// The saved instruction pointer points to the instruction that caused the `#PF`.
    ///
    /// The page-fault error code is described by the
    /// [`PageFaultErrorCode`](struct.PageFaultErrorCode.html) struct.
    ///
    /// The vector number of the `#PF` exception is 14.
    pub page_fault: Entry<PageFaultHandlerFunc>,

    /// vector nr. 15
    reserved_1: Entry<HandlerFunc>,

    /// The x87 Floating-Point Exception-Pending exception (`#MF`) is used to handle unmasked x87
    /// floating-point exceptions. In 64-bit mode, the x87 floating point unit is not used
    /// anymore, so this exception is only relevant when executing programs in the 32-bit
    /// compatibility mode.
    ///
    /// The vector number of the `#MF` exception is 16.
    pub x87_floating_point: Entry<HandlerFunc>,

    /// An alignment check exception (`#AC`) occurs when an unaligned-memory data reference
    /// is performed while alignment checking is enabled. An `#AC` can occur only when CPL=3.
    ///
    /// The returned error code is always zero. The saved instruction pointer points to the
    /// instruction that caused the `#AC`.
    ///
    /// The vector number of the `#AC` exception is 17.
    pub alignment_check: Entry<HandlerFuncWithErrCode>,

    /// The machine check exception (`#MC`) is model specific. Processor implementations
    /// are not required to support the `#MC` exception, and those implementations that do
    /// support `#MC` can vary in how the `#MC` exception mechanism works.
    ///
    /// There is no reliable way to restart the program.
    ///
    /// The vector number of the `#MC` exception is 18.
    pub machine_check: Entry<DivergingHandlerFunc>,

    /// The SIMD Floating-Point Exception (`#XF`) is used to handle unmasked SSE
    /// floating-point exceptions. The SSE floating-point exceptions reported by
    /// the `#XF` exception are (including mnemonics):
    ///
    /// - IE: Invalid-operation exception (also called #I).
    /// - DE: Denormalized-operand exception (also called #D).
    /// - ZE: Zero-divide exception (also called #Z).
    /// - OE: Overflow exception (also called #O).
    /// - UE: Underflow exception (also called #U).
    /// - PE: Precision exception (also called #P or inexact-result exception).
    ///
    /// The saved instruction pointer points to the instruction that caused the `#XF`.
    ///
    /// The vector number of the `#XF` exception is 19.
    pub simd_floating_point: Entry<HandlerFunc>,

    /// vector nr. 20
    pub virtualization: Entry<HandlerFunc>,

    /// A #CP exception is generated when shadow stacks are enabled and mismatch
    /// scenarios are detected (possible error code cases below).
    ///
    /// The error code is the #CP error code, for each of the following situations:
    /// - A RET (near) instruction encountered a return address mismatch.
    /// - A RET (far) instruction encountered a return address mismatch.
    /// - A RSTORSSP instruction encountered an invalid shadow stack restore token.
    /// - A SETSSBY instruction encountered an invalid supervisor shadow stack token.
    /// - A missing ENDBRANCH instruction if indirect branch tracking is enabled.
    ///
    /// vector nr. 21
    pub cp_protection_exception: Entry<HandlerFuncWithErrCode>,

    /// vector nr. 22-27
    reserved_2: [Entry<HandlerFunc>; 6],

    /// The Hypervisor Injection Exception (`#HV`) is injected by a hypervisor
    /// as a doorbell to inform an `SEV-SNP` enabled guest running with the
    /// `Restricted Injection` feature of events to be processed.
    ///
    /// `SEV-SNP` stands for the _"Secure Nested Paging"_ feature of the _"AMD
    /// Secure Encrypted Virtualization"_  technology. The `Restricted
    /// Injection` feature disables all hypervisor-based interrupt queuing
    /// and event injection of all vectors except #HV.
    ///
    /// The `#HV` exception is a benign exception and can only be injected as
    /// an exception and without an error code. `SEV-SNP` enabled guests are
    /// expected to communicate with the hypervisor about events via a
    /// software-managed para-virtualization interface.
    ///
    /// The vector number of the ``#HV`` exception is 28.
    pub hv_injection_exception: Entry<HandlerFunc>,

    /// The VMM Communication Exception (`#VC`) is always generated by hardware when an `SEV-ES`
    /// enabled guest is running and an `NAE` event occurs.
    ///
    /// `SEV-ES` stands for the _"Encrypted State"_ feature of the _"AMD Secure Encrypted Virtualization"_
    /// technology. `NAE` stands for an _"Non-Automatic Exit"_, which is an `VMEXIT` event that requires
    /// hypervisor emulation. See
    /// [this whitepaper](https://www.amd.com/system/files/TechDocs/Protecting%20VM%20Register%20State%20with%20SEV-ES.pdf)
    /// for an overview of the `SEV-ES` feature.
    ///
    /// The `#VC` exception is a precise, contributory, fault-type exception utilizing exception vector 29.
    /// This exception cannot be masked. The error code of the `#VC` exception is equal
    /// to the `#VMEXIT` code of the event that caused the `NAE`.
    ///
    /// In response to a `#VC` exception, a typical flow would involve the guest handler inspecting the error
    /// code to determine the cause of the exception and deciding what register state must be copied to the
    /// `GHCB` (_"Guest Hypervisor Communication Block"_) for the event to be handled. The handler
    /// should then execute the `VMGEXIT` instruction to
    /// create an `AE` and invoke the hypervisor. After a later `VMRUN`, guest execution will resume after the
    /// `VMGEXIT` instruction where the handler can view the results from the hypervisor and copy state from
    /// the `GHCB` back to its internal state as needed.
    ///
    /// Note that it is inadvisable for the hypervisor to set the `VMCB` (_"Virtual Machine Control Block"_)
    /// intercept bit for the `#VC` exception as
    /// this would prevent proper handling of `NAE`s by the guest. Similarly, the hypervisor should avoid
    /// setting intercept bits for events that would occur in the `#VC` handler (such as `IRET`).
    ///
    /// The vector number of the ``#VC`` exception is 29.
    pub vmm_communication_exception: Entry<HandlerFuncWithErrCode>,

    /// The Security Exception (`#SX`) signals security-sensitive events that occur while
    /// executing the VMM, in the form of an exception so that the VMM may take appropriate
    /// action. (A VMM would typically intercept comparable sensitive events in the guest.)
    /// In the current implementation, the only use of the `#SX` is to redirect external INITs
    /// into an exception so that the VMM may — among other possibilities.
    ///
    /// The only error code currently defined is 1, and indicates redirection of INIT has occurred.
    ///
    /// The vector number of the ``#SX`` exception is 30.
    pub security_exception: Entry<HandlerFuncWithErrCode>,

    /// vector nr. 31
    reserved_3: Entry<HandlerFunc>,

    /// User-defined interrupts can be initiated either by system logic or software. They occur
    /// when:
    ///
    /// - System logic signals an external interrupt request to the processor. The signaling
    ///   mechanism and the method of communicating the interrupt vector to the processor are
    ///   implementation dependent.
    /// - Software executes an `INTn` instruction. The `INTn` instruction operand provides
    ///   the interrupt vector number.
    ///
    /// Both methods can be used to initiate an interrupt into vectors 0 through 255. However,
    /// because vectors 0 through 31 are defined or reserved by the AMD64 architecture,
    /// software should not use vectors in this range for purposes other than their defined use.
    ///
    /// The saved instruction pointer depends on the interrupt source:
    ///
    /// - External interrupts are recognized on instruction boundaries. The saved instruction
    ///   pointer points to the instruction immediately following the boundary where the
    ///   external interrupt was recognized.
    /// - If the interrupt occurs as a result of executing the INTn instruction, the saved
    ///   instruction pointer points to the instruction after the INTn.
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

    /// Returns slice of IDT entries with the specified range.
    ///
    /// Panics if the entry is an exception.
    #[inline]
    pub fn slice(&self, bounds: impl RangeBounds<u8>) -> &[Entry<HandlerFunc>] {
        let (lower_idx, upper_idx) = self.condition_slice_bounds(bounds);
        &self.interrupts[(lower_idx - 32)..(upper_idx - 32)]
    }

    /// Returns a mutable slice of IDT entries with the specified range.
    ///
    /// Panics if the entry is an exception.
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

    /// Returns the IDT entry with the specified index.
    ///
    /// Panics if the entry is an exception that pushes an error code (use the struct fields for accessing these entries).
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
    /// Returns a mutable reference to the IDT entry with the specified index.
    ///
    /// Panics if the entry is an exception that pushes an error code (use the struct fields for accessing these entries).
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

            /// Returns the IDT entry with the specified index.
            ///
            /// Panics if index is outside the IDT (i.e. greater than 255) or if the entry is an
            /// exception that pushes an error code (use the struct fields for accessing these entries).
            #[inline]
            fn index(&self, index: $ty) -> &Self::Output {
                self.slice(index)
            }
        }

        impl IndexMut<$ty> for InterruptDescriptorTable {
            /// Returns a mutable reference to the IDT entry with the specified index.
            ///
            /// Panics if the entry is an exception that pushes an error code (use the struct fields for accessing these entries).
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

/// An Interrupt Descriptor Table entry.
///
/// The generic parameter is some [`HandlerFuncType`], depending on the interrupt vector.
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
    /// Creates a non-present IDT entry (but sets the must-be-one bits).
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

    /// Sets the handler address for the IDT entry and sets the following defaults:
    ///   - The code selector is the code segment currently active in the CPU
    ///   - The present bit is set
    ///   - Interrupts are disabled on handler invocation
    ///   - The privilege level (DPL) is [`PrivilegeLevel::Ring0`]
    ///   - No IST is configured (existing stack will be used)
    ///
    /// The function returns a mutable reference to the entry's options that allows
    /// further customization.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `addr` is the address of a valid interrupt handler function,
    /// and the signature of such a function is correct for the entry type.
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

    /// Returns the virtual address of this IDT entry's handler function.
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

/// A common trait for all handler functions usable in [`Entry`].
///
/// # Safety
///
/// Implementors have to ensure that `to_virt_addr` returns a valid address.
pub unsafe trait HandlerFuncType {
    /// Get the virtual address of the handler function.
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

/// Represents the 4 non-offset bytes of an IDT entry.
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
    /// Creates a minimal options field with all the must-be-one bits set. This
    /// means the CS selector, IST, and DPL field are all 0.
    #[inline]
    const fn minimal() -> Self {
        EntryOptions {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000, // Default to a 64-bit Interrupt Gate
        }
    }

    /// Set the code segment that will be used by this interrupt.
    ///
    /// ## Safety
    /// This function is unsafe because the caller must ensure that the passed
    /// segment selector points to a valid, long-mode code segment.
    pub unsafe fn set_code_selector(&mut self, cs: SegmentSelector) -> &mut Self {
        self.cs = cs;
        self
    }

    /// Set or reset the preset bit.
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

    /// Assigns a Interrupt Stack Table (IST) stack to this handler. The CPU will then always
    /// switch to the specified stack before the handler is invoked. This allows kernels to
    /// recover from corrupt stack pointers (e.g., on kernel stack overflow).
    ///
    /// An IST stack is specified by an IST index between 0 and 6 (inclusive). Using the same
    /// stack for multiple interrupts can be dangerous when nested interrupts are possible.
    ///
    /// This function panics if the index is not in the range 0..7.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must ensure that the passed stack index is
    /// valid and not used by other interrupts. Otherwise, memory safety violations are possible.
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

/// Wrapper type for the interrupt stack frame pushed by the CPU.
///
/// This type derefs to an [`InterruptStackFrameValue`], which allows reading the actual values.
///
/// This wrapper type ensures that no accidental modification of the interrupt stack frame
/// occurs, which can cause undefined behavior (see the [`as_mut`](InterruptStackFrame::as_mut)
/// method for more information).
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

/// Represents the interrupt stack frame pushed by the CPU on interrupt or exception entry.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    /// This value points to the instruction that should be executed when the interrupt
    /// handler returns. For most interrupts, this value points to the instruction immediately
    /// following the last executed instruction. However, for some exceptions (e.g., page faults),
    /// this value points to the faulting instruction, so that the instruction is restarted on
    /// return. See the documentation of the [`InterruptDescriptorTable`] fields for more details.
    pub instruction_pointer: VirtAddr,
    /// The code segment selector at the time of the interrupt.
    pub code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    /// The flags register before the interrupt handler was invoked.
    pub cpu_flags: RFlags,
    /// The stack pointer at the time of the interrupt.
    pub stack_pointer: VirtAddr,
    /// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
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
    /// Describes an page fault error code.
    ///
    /// This structure is defined by the following manual sections:
    ///   * AMD Volume 2: 8.4.2
    ///   * Intel Volume 3A: 4.7
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageFaultErrorCode: u64 {
        /// If this flag is set, the page fault was caused by a page-protection violation,
        /// else the page fault was caused by a not-present page.
        const PROTECTION_VIOLATION = 1;

        /// If this flag is set, the memory access that caused the page fault was a write.
        /// Else the access that caused the page fault is a memory read. This bit does not
        /// necessarily indicate the cause of the page fault was a read or write violation.
        const CAUSED_BY_WRITE = 1 << 1;

        /// If this flag is set, an access in user mode (CPL=3) caused the page fault. Else
        /// an access in supervisor mode (CPL=0, 1, or 2) caused the page fault. This bit
        /// does not necessarily indicate the cause of the page fault was a privilege violation.
        const USER_MODE = 1 << 2;

        /// If this flag is set, the page fault is a result of the processor reading a 1 from
        /// a reserved field within a page-translation-table entry.
        const MALFORMED_TABLE = 1 << 3;

        /// If this flag is set, it indicates that the access that caused the page fault was an
        /// instruction fetch.
        const INSTRUCTION_FETCH = 1 << 4;

        /// If this flag is set, it indicates that the page fault was caused by a protection key.
        const PROTECTION_KEY = 1 << 5;

        /// If this flag is set, it indicates that the page fault was caused by a shadow stack
        /// access.
        const SHADOW_STACK = 1 << 6;

        /// If this flag is set, it indicates that the page fault was caused by SGX access-control
        /// requirements (Intel-only).
        const SGX = 1 << 15;

        /// If this flag is set, it indicates that the page fault is a result of the processor
        /// encountering an RMP violation (AMD-only).
        const RMP = 1 << 31;
    }
}

/// Describes an error code referencing a segment selector.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SelectorErrorCode {
    flags: u64,
}

impl SelectorErrorCode {
    /// If true, indicates that the exception occurred during delivery of an event
    /// external to the program, such as an interrupt or an earlier exception.
    pub fn external(&self) -> bool {
        self.flags.get_bit(0)
    }

    /// The descriptor table this error code refers to.
    pub fn descriptor_table(&self) -> DescriptorTable {
        match self.flags.get_bits(1..3) {
            0b00 => DescriptorTable::Gdt,
            0b01 => DescriptorTable::Idt,
            0b10 => DescriptorTable::Ldt,
            0b11 => DescriptorTable::Idt,
            _ => unreachable!(),
        }
    }

    /// The index of the selector which caused the error.
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

/// The possible descriptor table values.
///
/// Used by the [`SelectorErrorCode`] to indicate which table caused the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorTable {
    /// Global Descriptor Table.
    Gdt,
    /// Interrupt Descriptor Table.
    Idt,
    /// Logical Descriptor Table.
    Ldt,
}
