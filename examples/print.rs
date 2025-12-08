#![feature(c_variadic)]
#![allow(unsafe_op_in_unsafe_fn)]
use lwprintf_rs::{
    CustomOutPut, LwprintfObj, lwprintf_init, lwprintf_init_ex, lwprintf_printf,
    lwprintf_printf_ex, lwprintf_sprintf, lwprintf_sprintf_ex, lwprintf_vprintf_ex,
    lwprintf_vprintf_ex_rust, lwprintf_vsnprintf_ex, lwprintf_vsnprintf_ex_rust,
};

struct StdOut;

impl CustomOutPut for StdOut {
    fn putch(ch: i32) -> i32 {
        print!("{}", ch as u8 as char);
        ch
    }
}

/// Demonstrate various lwprintf functions with a custom output function.
unsafe fn lwprintf_ex() {
    println!("--- lwprintf_ex demo ---");
    let mut lwobj = LwprintfObj::<StdOut>::new();
    // initialize lwprintf object with custom output function
    lwprintf_init_ex(&mut lwobj);

    lwprintf_printf_ex(
        lwobj.as_mut_ptr(),
        b"Hello, printf_ex Number: %d, String: %s\n\0".as_ptr() as *const i8,
        1 as i32,
        b"printf_ex\0".as_ptr(),
    );

    let mut buf = [0u8; 100];
    let l = lwprintf_rs::lwprintf_snprintf_ex(
        lwobj.as_mut_ptr(),
        buf.as_mut_ptr() as *mut i8,
        buf.len(),
        b"Hello, snprintf_ex Number: %d, String: %s\n\0".as_ptr() as *const i8,
        2 as i32,
        b"snprintf_ex\0".as_ptr(),
    );
    let s = core::str::from_utf8(&buf[..l as usize]).unwrap();
    print!("{}", s);

    let l = lwprintf_sprintf_ex!(
        lwobj.as_mut_ptr(),
        buf.as_mut_ptr() as *mut i8,
        b"Hello, sprintf_ex Number: %d, String: %s\n\0".as_ptr() as *const i8,
        3 as i32,
        b"sprintf_ex macro\0".as_ptr()
    );
    let s = core::str::from_utf8(&buf[..l as usize]).unwrap();
    print!("{}", s);

    unsafe extern "C" fn call_lwprintf_vprintf_ex(
        lwobj: &mut LwprintfObj<StdOut>,
        format: *const i8,
        mut args: ...
    ) {
        let args = args.as_va_list();
        unsafe {
            lwprintf_vprintf_ex(lwobj.as_mut_ptr(), format, args);
        }
    }

    call_lwprintf_vprintf_ex(
        &mut lwobj,
        b"Hello, vprintf_ex Number: %d, String: %s\n\0".as_ptr() as *const i8,
        4 as i32,
        b"vprintf_ex\0".as_ptr(),
    );

    unsafe extern "C" fn call_lwprintf_vsnprintf_ex(
        lwobj: &mut LwprintfObj<StdOut>,
        buf: *mut u8,
        n: usize,
        format: *const i8,
        mut args: ...
    ) -> i32 {
        let args = args.as_va_list();
        unsafe { lwprintf_vsnprintf_ex(lwobj.as_mut_ptr(), buf as *mut i8, n, format, args) }
    }

    let l = call_lwprintf_vsnprintf_ex(
        &mut lwobj,
        buf.as_mut_ptr(),
        buf.len(),
        b"Hello, vsnprintf_ex Number: %d, String: %s\n\0".as_ptr() as *const i8,
        6 as i32,
        b"vsnprintf_ex\0".as_ptr(),
    );
    let s = core::str::from_utf8(&buf[..l as usize]).unwrap();
    print!("{}", s);

    lwprintf_vprintf_ex_rust(
        lwobj.as_mut_ptr(),
        b"Hello, vprintf_ex_rust Number: %d, String: %s\n\0".as_ptr() as *const i8,
        5 as i32,
        b"vprintf_ex_rust\0".as_ptr(),
    );

    let l = lwprintf_vsnprintf_ex_rust(
        lwobj.as_mut_ptr(),
        buf.as_mut_ptr() as *mut i8,
        buf.len(),
        b"Hello, vsnprintf_ex_rust Number: %d, String: %s\n\0".as_ptr() as *const i8,
        6 as i32,
        b"vsnprintf_ex_rust\0".as_ptr(),
    );
    let s = core::str::from_utf8(&buf[..l as usize]).unwrap();
    print!("{}", s);
}

/// Print formatted data to the output with default LwPRINTF instance.
fn lwprintf() {
    println!("--- lwprintf demo ---");
    // initialize default lwprintf instance
    lwprintf_init::<StdOut>();
    lwprintf_printf!(
        b"Hello, printf Number: %d, String: %s\n\0".as_ptr() as *const i8,
        200 as i32,
        b"printf\0".as_ptr()
    );

    let mut buf = [0u8; 100];
    let l = lwprintf_sprintf!(
        buf.as_mut_ptr() as *mut i8,
        b"Hello, sprintf Number: %d, String: %s\n\0".as_ptr() as *const i8,
        300 as i32,
        b"sprintf macro\0".as_ptr()
    );
    let s = core::str::from_utf8(&buf[..l as usize]).unwrap();
    print!("{}", s);
    // other macros can be used similarly
}

fn other_test() {
    println!("--- other tests ---");
    lwprintf_printf!("float test: %.5f\n\0".as_ptr() as *const i8, 3.14159f64);
    lwprintf_printf!("hex test: 0x%08X\n\0".as_ptr() as *const i8, 0xDEADBEEFu32);
}

fn main() {
    unsafe {
        lwprintf_ex();
    }

    lwprintf();

    other_test();
}
