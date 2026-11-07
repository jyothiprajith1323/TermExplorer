#![no_std]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn on_command(_ptr: *const u8, _len: usize) -> i32 {
    let mut x: u64 = 0;
    loop {
        x = x.wrapping_add(1);
        core::hint::black_box(x);
    }
}
