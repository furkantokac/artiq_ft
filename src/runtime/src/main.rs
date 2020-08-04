#![no_std]
#![no_main]
#![recursion_limit="1024"]  // for futures_util::select!
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(c_variadic)]
#![feature(const_btree_new)]
#![feature(ptr_offset_from)]
#![feature(const_in_array_repeat_expressions)]
#![feature(naked_functions)]

extern crate alloc;

use core::{cmp, str};
use log::{info, warn, error};

use libboard_zynq::{timer::GlobalTimer, devc, slcr, mpcore, gic};
use libasync::{task, block_async};
use libsupport_zynq::ram;
use libregister::RegisterW;
use nb;
use void::Void;
use embedded_hal::blocking::delay::DelayMs;

mod sd_reader;
mod config;
mod net_settings;
mod proto_core_io;
mod proto_async;
mod comms;
mod rpc;
#[path = "../../../build/pl.rs"]
mod pl;
#[cfg(ki_impl = "csr")]
#[path = "rtio_csr.rs"]
mod rtio;
#[cfg(ki_impl = "acp")]
#[path = "rtio_acp.rs"]
mod rtio;
mod kernel;
mod moninj;
mod load_pl;
mod eh_artiq;
mod panic;
mod logger;
mod mgmt;
mod analyzer;
mod irq;

fn init_gateware() {
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
}

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

fn init_rtio(timer: &mut GlobalTimer, cfg: &config::Config) {
    let clock_sel =
        if let Ok(rtioclk) = cfg.read_str("rtioclk") {
            match rtioclk.as_ref() {
                "internal" => {
                    info!("using internal RTIO clock");
                    0
                },
                "external" => {
                    info!("using external RTIO clock");
                    1
                },
                other => {
                    warn!("RTIO clock specification '{}' not recognized", other);
                    info!("using internal RTIO clock");
                    0
                },
            }
        } else {
            info!("using internal RTIO clock (default)");
            0
        };

    loop {
        unsafe {
            pl::csr::rtio_crg::pll_reset_write(1);
            pl::csr::rtio_crg::clock_sel_write(clock_sel);
            pl::csr::rtio_crg::pll_reset_write(0);
        }
        timer.delay_ms(1);
        let locked = unsafe { pl::csr::rtio_crg::pll_locked_read() != 0 };
        if locked {
            info!("RTIO PLL locked");
            break;
        } else {
            warn!("RTIO PLL failed to lock, retrying...");
            timer.delay_ms(500);
        }
    }

    unsafe {
        pl::csr::rtio_core::reset_phy_write(1);
    }
}

fn wait_for_async_rtio_error() -> nb::Result<(), Void> {
    unsafe {
        if pl::csr::rtio_core::async_error_read() != 0 {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

async fn report_async_rtio_errors() {
    loop {
        let _ = block_async!(wait_for_async_rtio_error()).await;
        unsafe {
            let errors = pl::csr::rtio_core::async_error_read();
            if errors & 1 != 0 {
                error!("RTIO collision involving channel {}",
                       pl::csr::rtio_core::collision_channel_read());
            }
            if errors & 2 != 0 {
                error!("RTIO busy error involving channel {}",
                       pl::csr::rtio_core::busy_channel_read());
            }
            if errors & 4 != 0 {
                error!("RTIO sequence error involving channel {}",
                       pl::csr::rtio_core::sequence_error_channel_read());
            }
            pl::csr::rtio_core::async_error_write(errors);
        }
    }
}

static mut LOG_BUFFER: [u8; 1<<17] = [0; 1<<17];

#[no_mangle]
pub fn main_core0() {
    let mut timer = GlobalTimer::start();

    let buffer_logger = unsafe {
        logger::BufferLogger::new(&mut LOG_BUFFER[..])
    };
    buffer_logger.set_uart_log_level(log::LevelFilter::Debug);
    buffer_logger.register();
    log::set_max_level(log::LevelFilter::Debug);

    info!("NAR3/Zynq7000 starting...");

    ram::init_alloc_core0();
    gic::InterruptController::new(mpcore::RegisterBlock::new()).enable_interrupts();

    init_gateware();
    info!("detected gateware: {}", identifier_read(&mut [0; 64]));

    let cfg = match config::Config::new() {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!("config initialization failed: {}", err);
            config::Config::new_dummy()
        }
    };

    init_rtio(&mut timer, &cfg);
    task::spawn(report_async_rtio_errors());

    comms::main(timer, &cfg);
}
