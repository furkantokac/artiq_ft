[package]
authors = ["M-Labs"]
name = "satman"
version = "0.0.0"
build = "build.rs"

[features]
target_zc706 = ["libboard_zynq/target_zc706", "libsupport_zynq/target_zc706", "libconfig/target_zc706", "libboard_artiq/target_zc706"]
target_kasli_soc = ["libboard_zynq/target_kasli_soc", "libsupport_zynq/target_kasli_soc", "libconfig/target_kasli_soc", "libboard_artiq/target_kasli_soc"]
calibrate_wrpll_skew = ["libboard_artiq/calibrate_wrpll_skew"]
default = ["target_zc706", ]

[build-dependencies]
build_zynq = { path = "../libbuild_zynq" }

[dependencies]
log = { version = "0.4", default-features = false }
core_io = { version = "0.1", features = ["collections"] }
cslice = "0.3"
embedded-hal = "0.2"

libboard_zynq = { path = "@@ZYNQ_RS@@/libboard_zynq", features = ["ipv6"]}
libsupport_zynq = { path = "@@ZYNQ_RS@@/libsupport_zynq", default-features = false, features = ["alloc_core"] }
libcortex_a9 = { path = "@@ZYNQ_RS@@/libcortex_a9" }
libasync = { path = "@@ZYNQ_RS@@/libasync" }
libregister = { path = "@@ZYNQ_RS@@/libregister" }
libconfig = { path = "@@ZYNQ_RS@@/libconfig", features = ["fat_lfn", "ipv6"] }

libboard_artiq = { path = "../libboard_artiq" }
unwind = { path = "../libunwind" }
libc = { path = "../libc" }
io = { path = "../libio", features = ["alloc"] }
ksupport = { path = "../libksupport" }
