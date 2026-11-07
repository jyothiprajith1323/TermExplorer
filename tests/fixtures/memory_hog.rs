#![no_std]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn on_command(_ptr: *const u8, _len: usize) -> i32 {
    let res = core::arch::wasm32::memory_grow(0, 300);
    if res == usize::MAX {
        -1
    } else {
        42
    }
}
