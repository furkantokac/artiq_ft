#![no_std]
#![no_main]
#![recursion_limit="1024"]  // for futures_util::select!
#![feature(llvm_asm)]

extern crate alloc;

use core::{cmp, str};
use log::info;

use libboard_zynq::{timer::GlobalTimer, logger, devc};
use libsupport_zynq::ram;

mod sd_reader;
mod proto_core_io;
mod proto_async;
mod comms;
mod rpc;
#[path = "../../../build/pl.rs"]
mod pl;
mod rtio;
mod kernel;
mod moninj;


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

    let devc = devc::DevC::new();
    if devc.is_done() {
        info!("gateware already loaded");
        // Do not load again: assume that the gateware already present is
        // what we want (e.g. gateware configured via JTAG before PS
        // startup, or by FSBL).
    } else {
        info!("loading gateware");
        unimplemented!("gateware loading");
    }
    info!("detected gateware: {}", identifier_read(&mut [0; 64]));

    unsafe {
        pl::csr::rtio_core::reset_phy_write(1);
    }

    comms::main(timer);
}
