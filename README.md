# lwprintf-rs
Lightweight printf implementation for Rust, based on lwprintf C library.

## Problems

> Rust 编译器无法正确地在两个可变参数函数之间维护 ABI 兼容性,这是为什么?

原因在于 ABI 约定和编译器职责：

1. C 的可变参数是“调用者负责”协议：调用者要把每个实参按平台 ABI 约定放到寄存器/栈里；被调方用 va_list 依次取出。
2. Rust 仅允许在 extern "C" fn(... ) 上声明 C 可变参数，并直接把传入的真实实参按 C ABI 下发给对方。但 Rust 不支持把收到的 “...” 再次展开并转发给另一个可变参数函数，因为编译器无法重建原始的参数布局（哪些在寄存器、哪些在栈、宽度对齐等）。
3. 包装函数尝试 f(...args) 会制造错误的调用序列，栈/寄存器混乱，接收方解包时得到垃圾或崩溃