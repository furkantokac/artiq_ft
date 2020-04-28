#![no_std]
#![no_main]
#![recursion_limit="1024"]  // for futures_util::select!

extern crate alloc;
extern crate log;

use core::{cmp, str};
use log::info;

use libboard_zynq::timer::GlobalTimer;
use libsupport_zynq::{logger, ram};

mod proto;
mod comms;
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
    info!("NAR3 starting...");

    ram::init_alloc_linker();

    info!("Detected gateware: {}", identifier_read(&mut [0; 64]));

    unsafe {
        pl::csr::rtio_core::reset_phy_write(1);
    }

    comms::main(timer);
}
