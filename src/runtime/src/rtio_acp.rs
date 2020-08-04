use cslice::CSlice;

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

pub fn write_log(data: &[i8]) {
    unimplemented!();
}
