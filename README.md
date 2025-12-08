# lwprintf-rs
Lightweight printf bindings for Rust, powered by the upstream C library **lwprintf**. Provides `no_std` support and minimal glue to hook custom output sinks from Rust.

## Overview
- Builds the upstream C source via `cc` and generates bindings via `bindgen` at compile time.
- Exposes the core C APIs (`lwprintf_printf_ex`, `lwprintf_vprintf_ex`, `lwprintf_snprintf_ex`, `lwprintf_vsnprintf_ex`) plus Rust helpers and macros.
- Lets you plug in a Rust-defined output sink through the `CustomOutPut` trait and `LwprintfObj` wrapper.

## Public API surface (Rust crate)
- Trait: `CustomOutPut { fn putch(ch: i32) -> i32; }` – implement to handle each output byte/char.
- Struct: `LwprintfObj<T: CustomOutPut>`
	- `new()` – create an instance (uninitialized).
	- `as_mut_ptr()` – get `*mut lwprintf_t` for calling the raw C FFI.
- Functions (re-exported raw C FFI):
	- `lwprintf_printf_ex`, `lwprintf_vprintf_ex`
	- `lwprintf_snprintf_ex`, `lwprintf_vsnprintf_ex`
- Convenience wrappers (Rust-side varargs pass a `VaList` to the raw APIs):
	- `lwprintf_vprintf_ex_rust`, `lwprintf_vsnprintf_ex_rust`
- Macros (default instance = null `lwobj`):
	- `lwprintf_printf!`, `lwprintf_vprintf!`
	- `lwprintf_snprintf!`, `lwprintf_vsnprintf!`
	- `lwprintf_sprintf!`, `lwprintf_sprintf_ex!`

## Example usage
```shell
cargo run --example print
```

## Quick start
```rust
use lwprintf_rs::{CustomOutPut, LwprintfObj};

struct StdOut;
impl CustomOutPut for StdOut {
		fn putch(ch: i32) -> i32 { print!("{}", ch as u8 as char); ch }
}

fn main() {
		let mut lw = LwprintfObj::<StdOut>::new();
		lwprintf_rs::lwprintf_init_ex(&mut lw);
		unsafe {
				// Call the raw C varargs API using the object pointer
				lwprintf_rs::lwprintf_printf_ex(
						lw.as_mut_ptr(),
						b"Hello %s %d!\n\0".as_ptr() as *const i8,
						b"world\0".as_ptr() as *const i8,
						42,
				);
		}
}
```

## Notes on varargs
- Rust cannot safely re-forward C varargs; use the raw FFI exports for formatting (`lwprintf_printf_ex` etc.) and pass the pointer from `as_mut_ptr()`.
- For `va_list`-style calls, prefer `lwprintf_vprintf_ex` / `lwprintf_vsnprintf_ex` or the Rust helpers `lwprintf_vprintf_ex_rust` / `lwprintf_vsnprintf_ex_rust` which forward a `VaList` to C.

## Build
- The build script compiles `lwprintf.c` and generates bindings on the fly. No pre-generated bindings are checked in.
- Ensure `clang` and a C toolchain are available for bindgen/cc.

## Why varargs forwarding is tricky
Rust cannot preserve the original C varargs ABI layout when re-forwarding `...` across Rust functions. A Rust wrapper that takes `...` and then forwards to another `...` function will corrupt the call frame. Always call the raw C varargs functions directly (or use `va_list` variants) once arguments are marshalled.