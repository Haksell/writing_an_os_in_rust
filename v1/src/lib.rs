#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(exclusive_range_pattern)]

use core::{arch::asm, panic::PanicInfo};

mod entry;
mod idt;
mod interrupts;
mod pic;
mod port;
mod vga_buffer;

#[no_mangle]
pub extern "C" fn kernel_main() {
    println!("KERNEL");
    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: Yellow on Black
    println!("{}", info);
    hlt_loop()
}

fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}

#[inline]
fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}
