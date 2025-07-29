use {
    lazy_static::lazy_static,
    x86_64::{
        VirtAddr,
        structures::{
            gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
            tss::TaskStateSegment,
        },
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 5 << 12;
            // TODO: proper stack allocation
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(core::ptr::addr_of!(STACK) ) + STACK_SIZE as u64
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let selectors = Selectors {
            code_selector: gdt.append(Descriptor::kernel_code_segment()),
            tss_selector: gdt.append(Descriptor::tss_segment(&TSS)),
        };
        (gdt, selectors)
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        x86_64::instructions::tables::load_tss(GDT.1.tss_selector);
    }
}
