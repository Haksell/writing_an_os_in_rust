#![no_std]
#![allow(internal_features)]
#![feature(abi_x86_interrupt, allocator_api, ptr_internals, ptr_metadata)]
#![feature(asm_const, const_mut_refs, step_trait)] // TODO: remove?

#[macro_use]
mod vga_buffer;

mod asm;
mod interrupts;
mod memory;
mod multiboot;
mod xxx; // TODO: better name

extern crate alloc;

use self::{
    asm::{enable_nxe_bit, enable_write_protect_bit, hlt_loop},
    multiboot::MultiBoot,
};
use alloc::{string::String, vec};
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicUsize, Ordering},
};
use lazy_static::lazy_static;

lazy_static! {
    static ref MULTIBOOT: MultiBoot =
        unsafe { MultiBoot::load(MULTIBOOT_START.load(Ordering::SeqCst)) };
}

static MULTIBOOT_START: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_start: usize) {
    MULTIBOOT_START.store(multiboot_start, Ordering::SeqCst);

    // TODO: enable bits directly in asm?
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
