#![no_std]
#![allow(internal_features)]
#![feature(abi_x86_interrupt, allocator_api, ptr_internals)]

#[macro_use]
mod vga_buffer;

mod interrupts;
mod memory;
mod multiboot;

extern crate alloc;

use self::multiboot::MultiBoot;
use alloc::{string::String, vec};
use core::{
    arch::asm,
    panic::PanicInfo,
    sync::atomic::{AtomicUsize, Ordering},
};
use lazy_static::lazy_static;
use x86_64::registers::{
    control::{Cr0, Cr0Flags},
    model_specific::Msr,
};

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

fn enable_nxe_bit() {
    const IA32_EFER: u32 = 0xC0000080;
    const NXE_BIT: u64 = 1 << 11;

    let mut ia32_efer = Msr::new(IA32_EFER);
    unsafe {
        ia32_efer.write(ia32_efer.read() | NXE_BIT);
    }
}

fn enable_write_protect_bit() {
    unsafe {
        Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT);
    }
}
