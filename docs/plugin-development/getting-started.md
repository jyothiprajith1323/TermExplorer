# Plugin Development Walkthrough

This step-by-step walkthrough guides you through creating, compiling, and testing a TermExplorer WebAssembly plugin in 10 minutes.

---

## 1. Prerequisites

Ensure you have Rust and the WebAssembly compilation target installed:

```bash
# Add WebAssembly compilation target
rustup target add wasm32-unknown-unknown
```

---

## 2. Create the Plugin Project

Initialize a new library crate using Cargo:

```bash
cargo new --lib hello-telemetry
cd hello-telemetry
```

Update `Cargo.toml` to build a WebAssembly C-compatible dynamic library:

```toml
[package]
name = "hello-telemetry"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "z"        # Optimize for smallest binary size
lto = true
codegen-units = 1
panic = "abort"
```

---

## 3. Implement the Plugin Code (`src/lib.rs`)

Because WebAssembly plugins run in a lightweight sandbox without standard POSIX OS bindings, we use `#![no_std]`:

```rust
#![no_std]

// Host imported functions from the "env" namespace
extern "C" {
    fn host_log(ptr: *const u8, len: usize);
    fn host_set_status(ptr: *const u8, len: usize) -> i32;
    fn host_get_active_file(ptr: *mut u8, max_len: usize) -> i32;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static HELLO_MSG: &[u8] = b"Hello from WASM!";

#[no_mangle]
pub extern "C" fn init() -> i32 {
    unsafe {
        host_set_status(HELLO_MSG.as_ptr(), HELLO_MSG.len());
    }
    0
}

#[no_mangle]
pub extern "C" fn on_command(_cmd_ptr: *const u8, _cmd_len: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn get_status_text(out_ptr: *mut u8, max_len: usize) -> i32 {
    let len = HELLO_MSG.len().min(max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(HELLO_MSG.as_ptr(), out_ptr, len);
    }
    len as i32
}
```

---

## 4. Compile the WebAssembly Binary

Compile to the `wasm32-unknown-unknown` target:

```bash
cargo build --target wasm32-unknown-unknown --release
```

The resulting binary is located at `target/wasm32-unknown-unknown/release/hello_telemetry.wasm`.

---

## 5. Create the Plugin Manifest (`plugin.toml`)

Create `plugin.toml` alongside the `.wasm` file:

```toml
id = "hello-telemetry"
name = "Hello Telemetry Plugin"
version = "0.1.0"
author = "Developer"
description = "Demonstrates status telemetry and greeting"
entrypoint = "plugin.wasm"
permissions = ["status_text", "commands"]
```

Compute the SHA-256 hash of your binary and append it to `plugin.toml`:
```bash
sha256sum target/wasm32-unknown-unknown/release/hello_telemetry.wasm
```

Add the calculated hash to `plugin.toml`:
```toml
sha256 = "<calculated-hex-hash>"
```

---

## 6. Install and Test Locally

Copy your plugin into TermExplorer's local plugin directory:

```bash
mkdir -p ~/.local/share/termexplorer/plugins/hello-telemetry
cp target/wasm32-unknown-unknown/release/hello_telemetry.wasm ~/.local/share/termexplorer/plugins/hello-telemetry/plugin.wasm
cp plugin.toml ~/.local/share/termexplorer/plugins/hello-telemetry/plugin.toml
```

Launch TermExplorer:
```bash
termexplorer
```
Open **Plugin Center** (`Ctrl+Shift+X`) to verify your plugin is listed, active, and reporting status!
