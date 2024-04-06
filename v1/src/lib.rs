#![no_std]
#![allow(internal_features)]
#![feature(ptr_internals)]

#[macro_use]
mod vga_buffer;

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_information_address: usize) {
    vga_buffer::clear_screen();
    println!("ooga {}", 6 * 7);
    println!("{}", {
        println!("inner");
        "outer"
    });
    println!("ooga {}", 6 * 7);
    hlt_loop()
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    println!("{:?}", panic_info);
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
