#![no_std]

extern "C" {
    fn host_log(ptr: *const u8, len: usize);
    fn host_get_active_file(ptr: *mut u8, max_len: usize) -> i32;
    fn host_set_status(ptr: *const u8, len: usize) -> i32;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static STATUS_MSG: &[u8] = b"Inspector Ready";

#[no_mangle]
pub extern "C" fn init() -> i32 {
    unsafe {
        host_set_status(STATUS_MSG.as_ptr(), STATUS_MSG.len());
    }
    0
}

#[no_mangle]
pub extern "C" fn on_command(_cmd_ptr: *const u8, _cmd_len: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn get_status_text(out_ptr: *mut u8, max_len: usize) -> i32 {
    let len = STATUS_MSG.len().min(max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(STATUS_MSG.as_ptr(), out_ptr, len);
    }
    len as i32
}
