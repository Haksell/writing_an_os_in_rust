#![no_std]
#![allow(internal_features)]
#![feature(abi_x86_interrupt, allocator_api, ptr_internals, ptr_metadata)]

#[macro_use]
mod vga_buffer;

mod instructions;
mod interrupts;
mod memory;
mod multiboot;
mod structures;
mod virt_addr;

extern crate alloc;

use {
    self::{
        instructions::{enable_nxe_bit, enable_write_protect_bit, hlt_loop},
        multiboot::MultiBoot,
    },
    alloc::{string::String, vec},
    core::{
        panic::PanicInfo,
        sync::atomic::{AtomicUsize, Ordering},
    },
    lazy_static::lazy_static,
};

lazy_static! {
    static ref MULTIBOOT: MultiBoot =
        unsafe { MultiBoot::load(MULTIBOOT_START.load(Ordering::SeqCst)) };
}

static MULTIBOOT_START: AtomicUsize = AtomicUsize::new(0);

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(multiboot_start: usize) {
    MULTIBOOT_START.store(multiboot_start, Ordering::SeqCst);

    enable_nxe_bit();
    enable_write_protect_bit();

    vga_buffer::clear_screen();

    let mut memory_controller = memory::init();

    println!("This value is boxed: {}", *alloc::boxed::Box::new(42));
    println!("This string too: {}", String::from("ooga") + "chaka");
    println!("Fibonacci: {:?}", vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);

    interrupts::init(&mut memory_controller);

    println!("No crash! \x02");
    hlt_loop()
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    println!("{:?}", panic_info);
    hlt_loop()
}
