#![no_std]
#![no_main]
#![recursion_limit="1024"]  // for futures_util::select!
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

use core::{cmp, str};
use log::{info, warn};

use libboard_zynq::{timer::GlobalTimer, logger, devc, slcr};
use libsupport_zynq::ram;
use libregister::RegisterW;

mod sd_reader;
mod config;
mod net_settings;
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

    // Set up PS->PL clocks
    slcr::RegisterBlock::unlocked(|slcr| {
        // As we are touching the mux, the clock may glitch, so reset the PL.
        slcr.fpga_rst_ctrl.write(
            slcr::FpgaRstCtrl::zeroed()
                .fpga0_out_rst(true)
                .fpga1_out_rst(true)
                .fpga2_out_rst(true)
                .fpga3_out_rst(true)
        );
        slcr.fpga0_clk_ctrl.write(
            slcr::Fpga0ClkCtrl::zeroed()
                .src_sel(slcr::PllSource::IoPll)
                .divisor0(8)
                .divisor1(1)
        );
        slcr.fpga_rst_ctrl.write(
            slcr::FpgaRstCtrl::zeroed()
        );
    });
    if devc::DevC::new().is_done() {
        info!("gateware already loaded");
        // Do not load again: assume that the gateware already present is
        // what we want (e.g. gateware configured via JTAG before PS
        // startup, or by FSBL).
        // Make sure that the PL/PS interface is enabled (e.g. OpenOCD does not enable it).
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_postload_fpga();
        });
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

    let cfg = match config::Config::new() {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!("config initialization failed: {}", err);
            config::Config::new_dummy()
        }
    };
    comms::main(timer, &cfg);
}
