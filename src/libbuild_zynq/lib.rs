use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn cfg() {
    // Handle rustc-cfg file
    let cfg_path = "../../build/rustc-cfg";
    println!("cargo:rerun-if-changed={}", cfg_path);

    let f = BufReader::new(File::open(cfg_path).unwrap());
    for line in f.lines() {
        println!("cargo:rustc-cfg={}", line.unwrap());
    }
}
