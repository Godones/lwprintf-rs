use std::{env, path::PathBuf, process::Command};

fn main() {
    let sysroot = build_lib();
    generates_bindings_to_rust(sysroot.as_deref());
}

fn build_lib() -> Option<String> {
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

    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let libc_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let mut builder = cc::Build::new();
    builder
        .file(&c_src)
        .include(&include_dir)
        .include(&manifest_dir)
        .warnings(true);

    let get_sys_root = |cc: &str| {
        let output = Command::new(cc)
            .args(["-print-sysroot"])
            .output()
            .expect("failed to execute process: gcc -print-sysroot");

        let sysroot = core::str::from_utf8(&output.stdout).unwrap();
        let sysroot = sysroot.trim_end();
        format!("-I{}/include/", sysroot)
    };

    let sys_root = if os == "none" {
        let musl_gcc = format!("{}-linux-musl-gcc", arch);
        builder.flag_if_supported("-ffreestanding");
        builder.flag_if_supported("-nostdlib");
        builder.compiler(&musl_gcc);

        let sysroot = get_sys_root(&musl_gcc);
        Some(sysroot)
    } else if arch == "loongarch64" && libc_env == "musl" {
        let musl_gcc = format!("{}-linux-musl-gcc", arch);
        let sysroot = get_sys_root(&musl_gcc);
        Some(sysroot)
    } else {
        None
    };

    builder.compile("lwprintf");
    sys_root
}

fn generates_bindings_to_rust(sysroot: Option<&str>) {
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
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if os == "none" {
        // For bare-metal targets, use musl target for proper bindings generation
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let musl_target = format!("{}-unknown-linux-musl", arch);
        // replace the target environment variable with musl target
        unsafe {
            env::set_var("TARGET", musl_target);
        }
    }

    let mut bindings = bindgen::Builder::default()
        .use_core()
        .wrap_unsafe_ops(true)
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        // search for lwprintf_opts.h in crate root (provided by user)
        .clang_arg(format!("-I{}", opts_dir.display()))
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if let Some(sysroot) = sysroot {
        bindings = bindings.clang_arg(sysroot);
    }

    let bindings = bindings.generate().expect("Unable to generate bindings");

    if os == "none" {
        // restore the original target environment variable
        unsafe {
            env::set_var("TARGET", target);
        }
    }

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("lwprintf.rs"))
        .expect("Couldn't write bindings!");
}
