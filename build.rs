use std::{env, path::PathBuf};

fn main() {
    build_lib();
    generates_bindings_to_rust();
}

fn build_lib() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let c_src = manifest_dir.join("./lwprintf/lwprintf/src/lwprintf/lwprintf.c");
    let include_dir = manifest_dir.join("./lwprintf/lwprintf/src/include");
    let opts_file = manifest_dir.join("lwprintf_opts.h");

    println!("cargo:rerun-if-changed={}", c_src.display());
    println!("cargo:rerun-if-changed={}", opts_file.display());
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("lwprintf/lwprintf.h").display()
    );

    cc::Build::new()
        .file(&c_src)
        .include(&include_dir)
        .include(&manifest_dir)
        .warnings(true)
        .compile("lwprintf");
}

fn generates_bindings_to_rust() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header = manifest_dir.join("wrapper.h");
    let include_dir = manifest_dir.join("./lwprintf/lwprintf/src/include");
    let opts_dir = &manifest_dir;

    println!("cargo:rerun-if-changed={}", header.display());
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("lwprintf.h").display()
    );

    let target = env::var("TARGET").unwrap();
    if target.ends_with("-softfloat") {
        // Clang does not recognize the `-softfloat` suffix
        unsafe { env::set_var("TARGET", target.replace("-softfloat", "")) };
    }

    let bindings = bindgen::Builder::default()
        .use_core()
        .wrap_unsafe_ops(true)
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        // search for lwprintf_opts.h in crate root (provided by user)
        .clang_arg(format!("-I{}", opts_dir.display()))
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Restore the original target environment variable
    unsafe { env::set_var("TARGET", target) };

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("lwprintf.rs"))
        .expect("Couldn't write bindings!");
}
