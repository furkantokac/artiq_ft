#![no_std]
#![no_main]

extern crate log;

use core::mem;
use log::{debug, info, error};
use cstr_core::CStr;

use libcortex_a9::{enable_fpu, cache::dcci_slice};
use libboard_zynq::{
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
    logger,
    timer::GlobalTimer,
};
use libsupport_zynq as _;


extern "C" {
    fn unlzma_simple(buf: *const u8, in_len: i32,
                     output: *mut u8,
                     error: extern fn(*const u8)) -> i32;
}

extern fn lzma_error(message: *const u8) {
    error!("LZMA error: {}", unsafe { CStr::from_ptr(message) }.to_str().unwrap());
}

#[no_mangle]
pub fn main_core0() {
    GlobalTimer::start();
    logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    info!("Simple Zynq Loader starting...");

    enable_fpu();
    debug!("FPU enabled on Core0");

    const CPU_FREQ: u32 = 800_000_000;

    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart(); // reinitialize UART after clocking change
    let mut ddr = zynq::ddr::DdrRam::new();

    let payload = include_bytes!("../../../build/szl-payload.bin.lzma");
    info!("decompressing payload");
    let result = unsafe {
        unlzma_simple(payload.as_ptr(), payload.len() as i32, ddr.ptr(), lzma_error)
    };
    if result < 0 {
        error!("decompression failed");
    } else {
        // Flush data cache entries for all of DDR, including
        // Memory/Instruction Synchronization Barriers
        dcci_slice(unsafe {
            core::slice::from_raw_parts(ddr.ptr::<u8>(), ddr.size())
        });

        // Start core0 only, for compatibility with FSBL.
        info!("executing payload");
        unsafe {
            (mem::transmute::<*mut u8, fn()>(ddr.ptr::<u8>()))();
        }
    }

    loop {}
}

#[no_mangle]
pub fn main_core1() {
    panic!("core1 started but should not have");
}
