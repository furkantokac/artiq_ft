use core::ptr::{self, read_volatile, write_volatile};
use core::ffi::VaList;
use alloc::vec;
use cslice::CSlice;
use libc::{c_char, c_int, size_t};

use crate::artiq_raise;

use crate::pl::csr;


#[repr(C)]
pub struct TimestampedData {
    timestamp: i64,
    data: i32,
}

pub extern fn init() {
    unsafe {
        csr::rtio_core::reset_write(1);
    }
}

pub extern fn get_destination_status(destination: i32) -> bool {
    // TODO
    destination == 0
}

pub extern fn get_counter() -> i64 {
    unsafe {
        csr::rtio::counter_update_write(1);
        csr::rtio::counter_read() as i64
    }
}

pub extern fn now_mu() -> i64 {
    unimplemented!();
}

pub extern fn at_mu(t: i64) {
    unimplemented!();
}

pub extern fn delay_mu(dt: i64) {
    unimplemented!();
}

pub extern fn output(target: i32, data: i32) {
    unimplemented!();
}

pub extern fn output_wide(target: i32, data: CSlice<i32>) {
    unimplemented!();
}

pub extern fn input_timestamp(timeout: i64, channel: i32) -> i64 {
   unimplemented!();
}

pub extern fn input_data(channel: i32) -> i32 {
    unimplemented!();
}

pub extern fn input_timestamped_data(timeout: i64, channel: i32) -> TimestampedData {
    unimplemented!();
}

extern "C" {
    fn vsnprintf_(buffer: *mut c_char, count: size_t, format: *const c_char, va: VaList) -> c_int;
}

fn write_rtio_log(data: &[i8]) {
    unimplemented!();
}

pub unsafe extern fn log(fmt: *const c_char, mut args: ...) {
    let size = vsnprintf_(ptr::null_mut(), 0, fmt, args.as_va_list()) as usize;
    let mut buf = vec![0; size + 1];
    vsnprintf_(buf.as_mut_ptr(), size + 1, fmt, args.as_va_list());
    write_rtio_log(buf.as_slice());
}
