use std::process::Command;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // FIXME: this is dirty and unreliable. How to depend on the output of the runtime build?
    let payload = "../target/armv7-none-eabihf/release/runtime";

    let out = env::var("OUT_DIR").unwrap();
    let out_dir = &PathBuf::from(&out);
    let status = Command::new("llvm-objcopy")
                              .args(&["-O", "binary", payload, &format!("{}/payload.bin", out)])
                              .status().unwrap();
    assert!(status.success());
    let status = Command::new("lzma")
                              .args(&["--keep", "-f", &format!("{}/payload.bin", out)])
                              .status().unwrap();
    assert!(status.success());
    println!("cargo:rerun-if-changed={}", payload);

    let status = Command::new("clang")
                              .args(&["-target", "armv7-unknown-linux", "-fno-stack-protector",
                                      "src/unlzma.c", "-O2", "-c", "-fPIC", "-o",
                                      &format!("{}/unlzma.o", out)])
                              .status().unwrap();
    assert!(status.success());
    let status = Command::new("llvm-ar")
                              .args(&["crus", "libunlzma.a", "unlzma.o"])
                              .current_dir(&Path::new(&out))
                              .status().unwrap();
    assert!(status.success());
    println!("cargo:rustc-link-search=native={}", out);
    println!("cargo:rustc-link-lib=static=unlzma");
    println!("cargo:rerun-if-changed=src/unlzma.c");

    // Put the linker script somewhere the linker can find it
    File::create(out_dir.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Only re-run the build script when link.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=link.x");
}
