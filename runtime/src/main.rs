#![no_std]
#![no_main]

extern crate alloc;

use core::{cmp, str};

use libboard_zynq::{
    println,
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
};
use libsupport_zynq::{ram, boot};
use libcortex_a9::{mutex::Mutex, sync_channel::{self, sync_channel}};

mod comms;
mod pl;
mod rtio;
mod kernel;
mod control;


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

static mut STACK_CORE1: [u32; 512] = [0; 512];
static CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<usize>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<usize>>> = Mutex::new(None);

#[no_mangle]
pub fn main_core0() {
    println!("ARTIQ runtime starting...");

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

#[no_mangle]
pub fn main_core1() {
    println!("Core1 started");

    let mut core1_tx = None;
    while core1_tx.is_none() {
        core1_tx = CHANNEL_1TO0.lock().take();
    }
    let mut core1_tx = core1_tx.unwrap();

    let mut core1_rx = None;
    while core1_rx.is_none() {
        core1_rx = CHANNEL_0TO1.lock().take();
    }
    let mut core1_rx = core1_rx.unwrap();

    kernel::main(core1_tx, core1_rx);
}
