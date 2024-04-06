#![no_std]
#![no_main]

use blog_v2::{exit_qemu, hlt_loop, serial_print, serial_println, QemuExitCode, TEST_OK};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("should_panic ");
    assert_eq!(0, 1);
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("{}", TEST_OK);
    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}
