#![no_std]
#![no_main]

extern crate alloc;

use libboard_zynq::println;
use libsupport_zynq::ram;
use core::{cmp, str};

mod pl;

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
    println!("[CORE0] hello world {}", identifier_read(&mut [0; 64]));
    loop {}
}

#[no_mangle]
pub fn main_core1() {
    println!("[CORE1] hello world {}", identifier_read(&mut [0; 64]));
    loop {}
}
