#![no_std]
#![feature(never_type)]

extern crate core_io;
extern crate crc;
extern crate embedded_hal;
extern crate io;
extern crate libasync;
extern crate libboard_zynq;
extern crate libconfig;
extern crate libcortex_a9;
extern crate libregister;
extern crate log;
extern crate log_buffer;

pub mod drtio_routing;
#[cfg(has_drtio)]
pub mod drtioaux;
#[cfg(has_drtio)]
pub mod drtioaux_async;
pub mod drtioaux_proto;
#[cfg(feature = "target_kasli_soc")]
pub mod io_expander;
pub mod logger;
#[cfg(has_drtio)]
#[rustfmt::skip]
#[path = "../../../build/mem.rs"]
pub mod mem;
#[rustfmt::skip]
#[path = "../../../build/pl.rs"]
pub mod pl;
#[cfg(has_si5324)]
pub mod si5324;
#[cfg(has_drtio_eem)]
pub mod drtio_eem;

use core::{cmp, str};

pub fn identifier_read(buf: &mut [u8]) -> &str {
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
