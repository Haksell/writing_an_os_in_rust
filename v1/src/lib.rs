#![no_std]
#![allow(internal_features)]
#![feature(ptr_internals)]

mod vga_buffer;

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
pub extern "C" fn kernel_main() {
    vga_buffer::print_something();

    hlt_loop()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let hello = b"Panic Panic!";
    let color_byte = 0xe0;
    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }
    let buffer_ptr = (0xb8000 + 2308) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

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
