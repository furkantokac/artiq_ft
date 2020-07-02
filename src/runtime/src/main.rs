#![no_std]
#![no_main]
#![recursion_limit="1024"]  // for futures_util::select!
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

use core::{cmp, str};
use log::{info, error};

use libboard_zynq::{timer::GlobalTimer, logger, devc};
use libsupport_zynq::ram;

mod sd_reader;
mod config;
mod proto_core_io;
mod proto_async;
mod comms;
mod rpc;
#[path = "../../../build/pl.rs"]
mod pl;
mod rtio;
mod kernel;
mod moninj;
mod load_pl;
mod eh_artiq;
mod panic;

fn identifier_read(buf: &mut [u8]) -> &str {
    unsafe {
        pl::csr::identifier::address_write(0);
        let len = pl::csr::identifier::data_read();
        let len = cmp::min(len, buf.len() as u8);
        for i in 0..len {
            pl::csr::identifier::address_write(1 + i);
            buf[i as usize] = pl::csr::identifier::data_read();
        }
        str::from_utf8_unchecked(&buf[..len as usize])
    }
}

#[no_mangle]
pub fn main_core0() {
    let timer = GlobalTimer::start();
    let _ = logger::init();
    log::set_max_level(log::LevelFilter::Debug);
    info!("NAR3/Zynq7000 starting...");

    ram::init_alloc_linker();

    match config::Config::new() {
        Ok(mut cfg) => {
            match cfg.read_str("FOO") {
                Ok(val) => info!("FOO = {}", val),
                Err(error) => info!("failed to read config FOO: {}", error),
            }
            match cfg.read_str("BAR") {
                Ok(val) => info!("BAR = {}", val),
                Err(error) => info!("failed to read config BAR: {}", error),
            }
            match cfg.read_str("FOOBAR") {
                Ok(val) => info!("read FOOBAR = {}", val),
                Err(error) => info!("failed to read config FOOBAR: {}", error),
            }
        },
        Err(error) => error!("config failed: {}", error)
    }

    if devc::DevC::new().is_done() {
        info!("gateware already loaded");
        // Do not load again: assume that the gateware already present is
        // what we want (e.g. gateware configured via JTAG before PS
        // startup, or by FSBL).
    } else {
        // Load from SD card
        match load_pl::load_bitstream_from_sd() {
            Ok(_) => info!("Bitstream loaded successfully!"),
            Err(e) => info!("Failure loading bitstream: {}", e),
        }
    }
    info!("detected gateware: {}", identifier_read(&mut [0; 64]));

    unsafe {
        pl::csr::rtio_core::reset_phy_write(1);
    }

    comms::main(timer);
}
