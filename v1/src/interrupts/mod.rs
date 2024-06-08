mod gdt;

use crate::memory::MemoryController;
use gdt::Gdt;
use lazy_static::lazy_static;
use spin::Once;
use x86_64::{
    instructions::tables::load_tss,
    structures::{
        gdt::SegmentSelector,
        idt::{InterruptDescriptorTable, InterruptStackFrame},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
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
            VirtAddr::new(double_fault_stack.top() as u64);
        tss
    });

    // https://os.phil-opp.com/double-faults/#the-final-steps

    let (gdt, code_selector, tss_selector) = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        (
            gdt,
            gdt.add_entry(gdt::Descriptor::kernel_code_segment()),
            gdt.add_entry(gdt::Descriptor::tss_segment(&tss)),
        )
    });
    gdt.load();
    unsafe {
        set_cs(code_selector); // reload code segment register
        load_tss(*tss_selector);
    }
    println!("GDT loaded.");

    IDT.load();
    println!("IDT loaded.");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:?}", stack_frame);
    loop {}
}
