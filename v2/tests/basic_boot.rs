#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_v2::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_v2::{hlt_loop, println};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_v2::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
