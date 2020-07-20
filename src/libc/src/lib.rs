// Helper crate for dealing with c ffi
#![allow(non_camel_case_types)]
#![no_std]

use libboard_zynq::stdio;

pub type c_char = i8;
pub type c_int = i32;
pub type size_t = usize;
pub type uintptr_t = usize;
pub type c_void = core::ffi::c_void;

#[no_mangle]
extern "C" fn _putchar(byte: u8) {
    let mut uart = stdio::get_uart();
    uart.write_byte(byte);
}
