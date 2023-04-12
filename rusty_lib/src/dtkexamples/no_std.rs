#![no_std]
#![crate_type = "staticlib"]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn hello_world_app() -> i32 {
    0
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}