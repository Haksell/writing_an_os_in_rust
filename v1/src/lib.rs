#![no_std]
#![allow(internal_features)]
#![feature(ptr_internals)]

mod memory;
#[macro_use]
mod vga_buffer;

use core::{arch::asm, panic::PanicInfo};
use multiboot2::{BootInformationHeader, ElfSectionFlags};

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_start: usize) {
    vga_buffer::clear_screen();

    println!("multiboot_start: {:#X}", multiboot_start);
    let boot_info = unsafe {
        multiboot2::BootInformation::load(multiboot_start as *const BootInformationHeader).unwrap()
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

    let elf_sections = boot_info.elf_sections().unwrap();
    println!("kernel sections:");
    let mut count_sections = 0;
    let mut hidden_sections = 0;
    for section in elf_sections {
        if section.flags() != ElfSectionFlags::empty() {
            println!(
                "    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
                section.start_address(),
                section.size(),
                section.flags()
            );
        } else {
            hidden_sections += 1;
        }
        count_sections += 1;
    }
    println!(
        "{} total sections ({} hidden).",
        count_sections, hidden_sections
    );

    let kernel_start = boot_info
        .elf_sections()
        .unwrap()
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = boot_info
        .elf_sections()
        .unwrap()
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();
    let multiboot_end = multiboot_start + boot_info.total_size();

    println!(
        "kernel_start: {:#x}, kernel_end: {:#x}",
        kernel_start, kernel_end
    );
    println!(
        "multiboot_start: {:#x}, multiboot_end: {:#x}",
        multiboot_start, multiboot_end
    );

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
