#![no_std]
#![allow(internal_features)]
#![feature(ptr_internals)]

#[macro_use]
mod vga_buffer;

use core::{arch::asm, panic::PanicInfo};

use multiboot2::BootInformationHeader;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_information_address: usize) {
    vga_buffer::clear_screen();

    println!("{:#X}", multiboot_information_address);
    let boot_info = unsafe {
        multiboot2::BootInformation::load(
            multiboot_information_address as *const BootInformationHeader,
        )
        .unwrap()
    };
    let memory_map_tag = boot_info.memory_map_tag().unwrap();
    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!(
            "    start: 0x{:x}, length: 0x{:x}",
            area.start_address(),
            area.size()
        );
    }
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
