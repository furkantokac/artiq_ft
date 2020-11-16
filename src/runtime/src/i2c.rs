#[cfg(feature = "target_zc706")]
mod i2c {
    use libboard_zynq;
    use crate::artiq_raise;

    static mut I2C_BUS: Option<libboard_zynq::i2c::I2c> = None;

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

    pub fn init() {
        let mut i2c = libboard_zynq::i2c::I2c::i2c0();
        i2c.init().expect("I2C bus initialization failed");
        unsafe { I2C_BUS = Some(i2c) };
    }
}

#[cfg(not(feature = "target_zc706"))]
mod i2c {
    use crate::artiq_raise;

    pub extern fn start(_busno: i32) {
        artiq_raise!("I2CError", "No I2C bus");
    }

    pub extern fn restart(_busno: i32) {
        artiq_raise!("I2CError", "No I2C bus");
    }

    pub extern fn stop(_busno: i32) {
        artiq_raise!("I2CError", "No I2C bus");
    }

    pub extern fn write(_busno: i32, _data: i32) -> bool {
        artiq_raise!("I2CError", "No I2C bus");
    }

    pub extern fn read(_busno: i32, _ack: bool) -> i32 {
        artiq_raise!("I2CError", "No I2C bus");
    }

    pub fn init() {
    }
}

pub use i2c::*;
