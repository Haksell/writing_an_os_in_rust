#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(exclusive_range_pattern)]

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
pub extern "C" fn kernel_main() {
    let hello = b"Hello World!";
    let color_byte = 0x1f;
    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }
    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

    panic!("TEST");

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
