[package]
name = "libc"
version = "0.1.0"
authors = ["M-Labs"]
edition = "2018"
build = "build.rs"

[dependencies]
libboard_zynq = { path = "@@ZYNQ_RS@@/libboard_zynq" }

[build-dependencies]
cc = { version = "1.0.1" }
