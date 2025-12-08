# lwprintf-rs
轻量级的 Rust printf 绑定，基于上游 C 库 **lwprintf**。在 `no_std` 环境中工作，提供最小胶水以在 Rust 中接入自定义输出。

## 概览
- 编译期用 `cc` 构建 C 源码，并用 `bindgen` 生成绑定。
- 暴露 lwprintf 的核心 C 接口（`lwprintf_printf_ex` / `lwprintf_vprintf_ex` / `lwprintf_snprintf_ex` / `lwprintf_vsnprintf_ex`），并提供 Rust 侧的辅助宏与包装。
- 通过 `CustomOutPut` trait 和 `LwprintfObj`，可以用 Rust 定义输出目标（如串口、标准输出、日志缓冲等）。

## 公开接口（Rust crate）
- Trait: `CustomOutPut { fn putch(ch: i32) -> i32; }` —— 实现它以处理每个输出字节/字符。
- Struct: `LwprintfObj<T: CustomOutPut>`
  - `new()` —— 创建实例（尚未初始化）。
  - `as_mut_ptr()` —— 获取底层 `*mut lwprintf_t` 指针，可直接传给裸 FFI。
- 函数（重导出的裸 C FFI）：`lwprintf_printf_ex`、`lwprintf_vprintf_ex`、`lwprintf_snprintf_ex`、`lwprintf_vsnprintf_ex`
- Rust 侧 helper（把 `VaList` 传给 C）：`lwprintf_vprintf_ex_rust`、`lwprintf_vsnprintf_ex_rust`
- 宏（默认实例，即 `lwobj = null_mut()`）：
  - `lwprintf_printf!`、`lwprintf_vprintf!`
  - `lwprintf_snprintf!`、`lwprintf_vsnprintf!`
  - `lwprintf_sprintf!`、`lwprintf_sprintf_ex!`

## Example usage
```shell
cargo run --example print
```

## 快速上手示例
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
        // 直接调用 C 的可变参数接口，传入对象指针
        lwprintf_rs::lwprintf_printf_ex(
            lw.as_mut_ptr(),
            b"Hello %s %d!\n\0".as_ptr() as *const i8,
            b"world\0".as_ptr() as *const i8,
            42,
        );
    }
}
```

## 关于可变参数的注意事项
- Rust 无法安全地在“Rust 包装函数”里再次转发 C 的 `...`；这样会破坏 ABI 布局。应直接调用裸 C varargs 接口，或使用 `va_list` 版本（`lwprintf_vprintf_ex` / `lwprintf_vsnprintf_ex`），或使用提供的 Rust helper（将 `VaList` 传给 C）。

## 构建说明
- build.rs 会编译 `lwprintf.c` 并即时生成绑定，仓库不提交预生成绑定。
- 需要可用的 `clang` 与 C 工具链以支持 bindgen 和 cc 编译。

## 为什么不能在 Rust 里转发 C 可变参数
- C varargs 由调用者按平台 ABI 把参数放入寄存器/栈，callee 用 `va_list` 取出。
- Rust 允许在 `extern "C" fn(... )` 上声明 varargs，但无法重建并再次转发原始布局；包装函数尝试再调用另一 varargs 函数会导致栈/寄存器错位，输出异常或崩溃。
- 因此：在 Rust 中如需 varargs，请直接调用裸 C 接口，或改用 `va_list` 版本做桥接。
