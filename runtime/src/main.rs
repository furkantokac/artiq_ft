#![no_std]
#![no_main]

extern crate alloc;
extern crate log;

use core::{cmp, str};

use libboard_zynq::{
    println,
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
};
use libsupport_zynq::{logger, ram};

mod comms;
mod pl;
mod rtio;
mod kernel;


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
    println!("ARTIQ runtime starting...");
    let _ = logger::init();
    log::set_max_level(log::LevelFilter::Debug);

    const CPU_FREQ: u32 = 800_000_000;

    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart(); // reinitialize UART after clocking change
    let mut ddr = zynq::ddr::DdrRam::new();
    ram::init_alloc(&mut ddr);

    println!("Detected gateware: {}", identifier_read(&mut [0; 64]));

    comms::main();
}
