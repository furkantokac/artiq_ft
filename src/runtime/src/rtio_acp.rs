use cslice::CSlice;
use vcell::VolatileCell;
use libcortex_a9::asm;

use crate::artiq_raise;

use crate::pl::csr;

pub const RTIO_O_STATUS_WAIT:                      i32 = 1;
pub const RTIO_O_STATUS_UNDERFLOW:                 i32 = 2;
pub const RTIO_O_STATUS_DESTINATION_UNREACHABLE:   i32 = 4;
pub const RTIO_I_STATUS_WAIT_EVENT:                i32 = 1;
pub const RTIO_I_STATUS_OVERFLOW:                  i32 = 2;
pub const RTIO_I_STATUS_WAIT_STATUS:               i32 = 4;
pub const RTIO_I_STATUS_DESTINATION_UNREACHABLE:   i32 = 8;

#[repr(C)]
pub struct TimestampedData {
    timestamp: i64,
    data: i32,
}

#[repr(C, align(32))]
struct Transaction {
    request_cmd: i8,
    padding0: i8,
    padding1: i8,
    padding2: i8,
    request_target: i32,
    request_timestamp: i64,
    request_data: i64,
    padding: i64,
    reply_status: VolatileCell<i32>,
    reply_data: VolatileCell<i32>,
    reply_timestamp: VolatileCell<u64>
}

static mut TRANSACTION_BUFFER: Transaction = Transaction {
    request_cmd: 0,
    padding0: 0,
    padding1: 0,
    padding2: 0,
    request_target: 0,
    request_timestamp: 0,
    request_data: 0,
    padding: 0,
    reply_status: VolatileCell::new(0),
    reply_data: VolatileCell::new(0),
    reply_timestamp: VolatileCell::new(0)
};

pub extern fn init() {
    unsafe {
        csr::rtio_core::reset_write(1);
        csr::rtio::engine_addr_base_write(&TRANSACTION_BUFFER as *const Transaction as u32);
        csr::rtio::enable_write(1);
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
    unsafe { TRANSACTION_BUFFER.request_timestamp }
}

pub extern fn at_mu(t: i64) {
    unsafe { TRANSACTION_BUFFER.request_timestamp = t }
}

pub extern fn delay_mu(dt: i64) {
    unsafe { TRANSACTION_BUFFER.request_timestamp += dt }
}

#[inline(never)]
unsafe fn process_exceptional_status(channel: i32, status: i32) {
    let timestamp = now_mu();
    if status & RTIO_O_STATUS_WAIT != 0 {
        // FIXME: this is a kludge and probably buggy (kernel interrupted?)
        while csr::rtio::o_status_read() as i32 & RTIO_O_STATUS_WAIT != 0 {}
    }
    if status & RTIO_O_STATUS_UNDERFLOW != 0 {
        artiq_raise!("RTIOUnderflow",
            "RTIO underflow at {0} mu, channel {1}, slack {2} mu",
            timestamp, channel as i64, timestamp - get_counter());
    }
    if status & RTIO_O_STATUS_DESTINATION_UNREACHABLE != 0 {
        artiq_raise!("RTIODestinationUnreachable",
            "RTIO destination unreachable, output, at {0} mu, channel {1}",
            timestamp, channel as i64, 0);
    }
}

pub extern fn output(target: i32, data: i32) {
    unsafe {
        // Clear status so we can observe response
        TRANSACTION_BUFFER.reply_status.set(0);

        TRANSACTION_BUFFER.request_cmd = 0;
        TRANSACTION_BUFFER.request_target = target;
        TRANSACTION_BUFFER.request_data = data as i64;

        asm::dmb();
        asm::sev();

        let mut status;
        loop {
            status = TRANSACTION_BUFFER.reply_status.get();
            if status != 0 {
                break
            }
        }

        let status = status & !0x10000;
        if status != 0 {
            process_exceptional_status(target >> 8, status);
        }
    }
}

pub extern fn output_wide(target: i32, data: CSlice<i32>) {
    // TODO
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
    // TODO
    unimplemented!();
}
