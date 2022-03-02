use libboard_zynq;
use crate::artiq_raise;

pub static mut I2C_BUS: Option<libboard_zynq::i2c::I2c> = None;

pub extern fn start(busno: i32) {
    if busno > 0 {
        artiq_raise!("I2CError", "I2C bus could not be accessed");
    }
    unsafe {
        if (&mut I2C_BUS).as_mut().unwrap().start().is_err() {
            artiq_raise!("I2CError", "I2C start failed");
        }
    }
}

pub extern fn restart(busno: i32) {
    if busno > 0 {
        artiq_raise!("I2CError", "I2C bus could not be accessed");
    }
    unsafe {
        if (&mut I2C_BUS).as_mut().unwrap().restart().is_err() {
            artiq_raise!("I2CError", "I2C restart failed");
        }
    }
}

pub extern fn stop(busno: i32) {
    if busno > 0 {
        artiq_raise!("I2CError", "I2C bus could not be accessed");
    }
    unsafe {
        if (&mut I2C_BUS).as_mut().unwrap().stop().is_err() {
            artiq_raise!("I2CError", "I2C stop failed");
        }
    }
}

pub extern fn write(busno: i32, data: i32) -> bool {
    if busno > 0 {
        artiq_raise!("I2CError", "I2C bus could not be accessed");
    }
    unsafe {
        match (&mut I2C_BUS).as_mut().unwrap().write(data as u8) {
            Ok(r) => r,
            Err(_) => artiq_raise!("I2CError", "I2C write failed"),
        }
    }
}

pub extern fn read(busno: i32, ack: bool) -> i32 {
    if busno > 0 {
        artiq_raise!("I2CError", "I2C bus could not be accessed");
    }
    unsafe {
        match (&mut I2C_BUS).as_mut().unwrap().read(ack) {
            Ok(r) => r as i32,
            Err(_) => artiq_raise!("I2CError", "I2C read failed"),
        }
    }
}

pub extern fn switch_select(busno: i32, address: i32, mask: i32) {
    if busno > 0 {
        artiq_raise!("I2CError", "I2C bus could not be accessed");
    }
    let ch = match mask { //decode from mainline, PCA9548-centric API
        0x00 => None,
        0x01 => Some(0),
        0x02 => Some(1),
        0x04 => Some(2),
        0x08 => Some(3),
        0x10 => Some(4),
        0x20 => Some(5),
        0x40 => Some(6),
        0x80 => Some(7),
        _ => artiq_raise!("I2CError", "switch select supports only one channel")
    };
    unsafe {
        if (&mut I2C_BUS).as_mut().unwrap().pca954x_select(address as u8, ch).is_err() {
            artiq_raise!("I2CError", "switch select failed");
        }
    }
}

pub fn init() {
    let mut i2c = libboard_zynq::i2c::I2c::i2c0();
    i2c.init().expect("I2C bus initialization failed");
    unsafe { I2C_BUS = Some(i2c) };
}
