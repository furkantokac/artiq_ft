#![no_std]
#![no_main]

extern crate log;

use core::mem;
use log::{info, error};
use cstr_core::CStr;

use libboard_zynq::{
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
    timer::GlobalTimer,
};
use libsupport_zynq::{boot, logger};


static mut STACK_CORE1: [u32; 512] = [0; 512];

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
    let _ = logger::init();
    log::set_max_level(log::LevelFilter::Debug);
    info!("Simple Zynq Loader starting");

    const CPU_FREQ: u32 = 800_000_000;

    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart(); // reinitialize UART after clocking change
    let mut ddr = zynq::ddr::DdrRam::new();

    let payload = include_bytes!(concat!(env!("OUT_DIR"), "/payload.bin.lzma"));
    info!("decompressing payload");
    let result = unsafe {
        unlzma_simple(payload.as_ptr(), payload.len() as i32, ddr.ptr(), lzma_error)
    };
    if result < 0 {
        error!("decompression failed");
    } else {
        let core1_stack = unsafe { &mut STACK_CORE1[..] };
        boot::Core1::start(core1_stack);
        info!("executing payload");
        unsafe {
            (mem::transmute::<*mut u8, fn()>(ddr.ptr::<u8>()))();
        }
    }

    loop {}
}

#[no_mangle]
pub fn main_core1() {
    unsafe {
        (mem::transmute::<u32, fn()>(0x00100000))();
    }
}
