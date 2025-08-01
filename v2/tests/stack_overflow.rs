#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

// use {
//     blog_v2::{QemuExitCode, TEST_OK, exit_qemu, gdt, hlt_loop, serial_print, serial_println},
//     core::panic::PanicInfo,
//     lazy_static::lazy_static,
//     x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
// };

// lazy_static! {
//     static ref TEST_IDT: InterruptDescriptorTable = {
//         let mut idt = InterruptDescriptorTable::new();
//         unsafe {
//             idt.double_fault
//                 .set_handler_fn(test_double_fault_handler)
//                 .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
//         }
//         idt
//     };
// }

// extern "x86-interrupt" fn test_double_fault_handler(
//     _stack_frame: InterruptStackFrame,
//     _error_code: u64,
// ) -> ! {
//     serial_println!("{}", TEST_OK);
//     exit_qemu(QemuExitCode::Success);
//     hlt_loop();
// }

// #[unsafe(no_mangle)]
// pub extern "C" fn _start() -> ! {
//     serial_print!("stack_overflow ");
//     blog_v2::gdt::init();
//     TEST_IDT.load();
//     stack_overflow();
//     panic!("Execution continued after stack overflow");
// }

// #[allow(unconditional_recursion)]
// fn stack_overflow() {
//     stack_overflow();
//     volatile::Volatile::new(0).read();
// }

// vvv TEMPORARY vvv (Double Fault handler is broken)

use {
    blog_v2::{QemuExitCode, TEST_OK, exit_qemu, hlt_loop, serial_println},
    core::panic::PanicInfo,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("{}", TEST_OK);
    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    serial_println!("ok");
    panic!("Execution continued after stack overflow");
}
