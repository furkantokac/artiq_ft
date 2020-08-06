use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = env::var("OUT_DIR").unwrap();
    let out_dir = &PathBuf::from(&out);

    compile_unlzma();
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

pub fn compile_unlzma() {
    let cfg = &mut cc::Build::new();
    cfg.compiler("clang");
    cfg.no_default_flags(true);
    cfg.warnings(false);

    cfg.flag("-nostdlib");
    cfg.flag("-ffreestanding");
    cfg.flag("-fPIC");
    cfg.flag("-fno-stack-protector");
    cfg.flag("--target=armv7-none-eabihf");
    cfg.flag("-Os");

    let sources = vec![
        "unlzma.c",
    ];

    let root = Path::new("./");
    for src in sources {
        println!("cargo:rerun-if-changed={}", src);
        cfg.file(root.join("src").join(src));
    }

    cfg.compile("unlzma");
}
