#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_v2::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_v2::{
    allocator,
    memory::{self, BootInfoFrameAllocator},
    println,
    task::{executor::Executor, keyboard, Task},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("KFS {}", 6 * 7);
    blog_v2::init();

    let mut mapper = unsafe { memory::init(VirtAddr::new(boot_info.physical_memory_offset)) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_v2::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_v2::test_panic_handler(info)
}
