#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use blog_v2::{exit_qemu, gdt, hlt_loop, serial_print, serial_println, QemuExitCode, TEST_OK};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("{}", TEST_OK);
    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow ");
    blog_v2::gdt::init();
    TEST_IDT.load();
    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_v2::test_panic_handler(info)
}
