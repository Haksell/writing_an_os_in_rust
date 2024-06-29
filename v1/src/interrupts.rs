use crate::asm::{cs_set_reg, load_tss};
use crate::memory::MemoryController;
use crate::structures::gdt::{Descriptor, Gdt};
use crate::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::structures::tss::TaskStateSegment;
use crate::structures::SegmentSelector;
use crate::virt_addr::VirtAddr;
use lazy_static::lazy_static;
use spin::Once;

const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt
    };
}

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<(Gdt, SegmentSelector, SegmentSelector)> = Once::new();

pub fn init(memory_controller: &mut MemoryController) {
    let double_fault_stack = memory_controller
        .alloc_stack(1)
        .expect("could not allocate double fault stack");
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            VirtAddr::new(double_fault_stack.top as u64);
        tss
    });

    let (gdt, code_selector, tss_selector) = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&tss));
        (gdt, code_selector, tss_selector)
    });
    gdt.load();
    unsafe {
        cs_set_reg(*code_selector);
        load_tss(*tss_selector);
    }
    println!("GDT loaded.");

    IDT.load();
    println!("IDT loaded.");
}

// TODO: reimplement Debug for InterruptStackFrame

extern "x86-interrupt" fn breakpoint_handler(_: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
}

extern "x86-interrupt" fn double_fault_handler(_: InterruptStackFrame, _: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    loop {}
}