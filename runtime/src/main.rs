#![no_std]
#![no_main]

extern crate alloc;

use libboard_zynq::println;
use libsupport_zynq::ram;

#[no_mangle]
pub fn main_core0() {
    println!("hello world 000");
    loop {}
}

#[no_mangle]
pub fn main_core1() {
    println!("hello world 111");
    loop {}
}
