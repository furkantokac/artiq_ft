fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    libc::compile();
}

mod libc {
    use std::path::Path;
    pub fn compile() {
        let cfg = &mut cc::Build::new();
        cfg.cpp(false);
        cfg.warnings(false);

        // still have problem compiling the libunwind
        cfg.flag("-nostdlib");
        cfg.flag("-ffreestanding");
        cfg.flag("-fno-PIC");
        cfg.flag("-isystem../include");
        cfg.flag("-fno-stack-protector");
        cfg.flag("--target=armv7-none-eabihf");

        cfg.flag("-std=c99");
        cfg.flag("-fstrict-aliasing");
        cfg.flag("-funwind-tables");
        cfg.flag("-fvisibility=hidden");
        cfg.flag("-U_FORTIFY_SOURCE");
        cfg.define("_FORTIFY_SOURCE", Some("0"));

        let unwind_sources = vec![
            "printf.c"
        ];

        let root = Path::new("../libc");
        for src in unwind_sources {
            println!("cargo:rerun-if-changed={}", src);
            cfg.file(root.join("src").join(src));
        }

        cfg.compile("printf");
    }
}
