#![no_std]
#![allow(internal_features)]
#![feature(ptr_internals)]

#[macro_use]
mod vga_buffer;
mod memory;

use core::{arch::asm, panic::PanicInfo};
use multiboot2::{BootInformationHeader, ElfSectionFlags};
use x86_64::structures::paging::frame;

use crate::memory::FrameAllocator;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_start: usize) {
    vga_buffer::clear_screen();

    println!("multiboot_start: {:#X}", multiboot_start);
    let boot_info = unsafe {
        multiboot2::BootInformation::load(multiboot_start as *const BootInformationHeader).unwrap()
    };

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

    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        boot_info.memory_map_tag().unwrap().memory_areas(),
    );
    memory::remap_the_kernel(&mut frame_allocator, &boot_info);
    frame_allocator.allocate_frame();
    println!("kernel remapped! whatever that means.");

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
