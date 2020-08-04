use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when link.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=link.x");

    // Handle rustc-cfg file
    let cfg_path = "../../build/rustc-cfg";
    println!("cargo:rerun-if-changed={}", cfg_path);

    let f = BufReader::new(File::open(cfg_path).unwrap());
    for line in f.lines() {
        println!("cargo:rustc-cfg={}", line.unwrap());
    }
}
