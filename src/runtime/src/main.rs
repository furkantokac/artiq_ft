#![no_std]
#![no_main]
#![recursion_limit = "1024"] // for futures_util::select!
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(panic_info_message)]

#[macro_use]
extern crate alloc;

#[cfg(feature = "target_kasli_soc")]
use core::cell::RefCell;

use ksupport;
use libasync::task;
#[cfg(has_drtio_eem)]
use libboard_artiq::drtio_eem;
#[cfg(feature = "target_kasli_soc")]
use libboard_artiq::io_expander;
use libboard_artiq::{identifier_read, logger, pl};
use libboard_zynq::{gic, mpcore, timer::GlobalTimer};
use libconfig::Config;
use libcortex_a9::l2c::enable_l2_cache;
use libsupport_zynq::ram;
use log::{info, warn};

mod analyzer;
mod comms;

mod mgmt;
mod moninj;
mod panic;
mod proto_async;
mod rpc_async;
mod rtio_clocking;
mod rtio_dma;
mod rtio_mgt;
#[cfg(has_drtio)]
mod subkernel;

#[cfg(feature = "target_kasli_soc")]
async fn io_expanders_service(
    i2c_bus: RefCell<&mut libboard_zynq::i2c::I2c>,
    io_expander0: RefCell<io_expander::IoExpander>,
    io_expander1: RefCell<io_expander::IoExpander>,
) {
    loop {
        task::r#yield().await;
        io_expander0
            .borrow_mut()
            .service(&mut i2c_bus.borrow_mut())
            .expect("I2C I/O expander #0 service failed");
        io_expander1
            .borrow_mut()
            .service(&mut i2c_bus.borrow_mut())
            .expect("I2C I/O expander #1 service failed");
    }
}

#[cfg(has_grabber)]
mod grabber {
    use libasync::delay;
    use libboard_artiq::grabber;
    use libboard_zynq::time::Milliseconds;
    pub async fn grabber_thread(mut timer: GlobalTimer) {
        let mut countdown = timer.countdown();
        loop {
            grabber::tick();
            delay(&mut countdown, Milliseconds(200)).await;
        }
    }
}

static mut LOG_BUFFER: [u8; 1 << 17] = [0; 1 << 17];

#[no_mangle]
pub fn main_core0() {
    enable_l2_cache(0x8);
    let mut timer = GlobalTimer::start();

    let buffer_logger = unsafe { logger::BufferLogger::new(&mut LOG_BUFFER[..]) };
    buffer_logger.set_uart_log_level(log::LevelFilter::Info);
    buffer_logger.register();
    log::set_max_level(log::LevelFilter::Info);

    info!("NAR3/Zynq7000 starting...");

    ram::init_alloc_core0();
    gic::InterruptController::gic(mpcore::RegisterBlock::mpcore()).enable_interrupts();

    info!("gateware ident: {}", identifier_read(&mut [0; 64]));

    ksupport::i2c::init();
    #[cfg(feature = "target_kasli_soc")]
    let i2c_bus = unsafe { (ksupport::i2c::I2C_BUS).as_mut().unwrap() };

    #[cfg(feature = "target_kasli_soc")]
    let (mut io_expander0, mut io_expander1);
    #[cfg(feature = "target_kasli_soc")]
    {
        io_expander0 = io_expander::IoExpander::new(i2c_bus, 0).unwrap();
        io_expander1 = io_expander::IoExpander::new(i2c_bus, 1).unwrap();
        io_expander0
            .init(i2c_bus)
            .expect("I2C I/O expander #0 initialization failed");
        io_expander1
            .init(i2c_bus)
            .expect("I2C I/O expander #1 initialization failed");
        // Drive TX_DISABLE to false on SFP0..3
        io_expander0.set(0, 1, false);
        io_expander1.set(0, 1, false);
        io_expander0.set(1, 1, false);
        io_expander1.set(1, 1, false);
        io_expander0.service(i2c_bus).unwrap();
        io_expander1.service(i2c_bus).unwrap();
    }

    let cfg = match Config::new() {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!("config initialization failed: {}", err);
            Config::new_dummy()
        }
    };

    rtio_clocking::init(&mut timer, &cfg);

    #[cfg(has_drtio_eem)]
    drtio_eem::init(&mut timer, &cfg);

    #[cfg(has_grabber)]
    task::spawn(grabber::grabber_thread(timer));

    task::spawn(ksupport::report_async_rtio_errors());

    #[cfg(feature = "target_kasli_soc")]
    task::spawn(io_expanders_service(
        RefCell::new(i2c_bus),
        RefCell::new(io_expander0),
        RefCell::new(io_expander1),
    ));
    comms::main(timer, cfg);
}
