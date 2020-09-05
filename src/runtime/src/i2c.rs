use libboard_zynq;

static mut I2C_BUS: Option<libboard_zynq::i2c::I2c> = None;

pub extern fn start(_busno: i32) {
    unsafe {
        (&mut I2C_BUS).as_mut().unwrap().start().expect("I2C start failed")
    }
}

pub extern fn restart(_busno: i32) {
    unsafe {
        (&mut I2C_BUS).as_mut().unwrap().restart().expect("I2C restart failed")
    }
}

pub extern fn stop(_busno: i32) {
    unsafe {
        (&mut I2C_BUS).as_mut().unwrap().stop().expect("I2C stop failed")
    }
}

pub extern fn write(_busno: i32, data: i32) -> bool {
    unsafe {
        (&mut I2C_BUS).as_mut().unwrap().write(data as u8).expect("I2C write failed")
    }
}

pub extern fn read(_busno: i32, ack: bool) -> i32 {
    unsafe {
        (&mut I2C_BUS).as_mut().unwrap().read(ack).expect("I2C read failed") as i32
    }
}

pub fn init() {
    let mut i2c = libboard_zynq::i2c::I2c::i2c0();
    i2c.init().expect("I2C bus initialization failed");
    unsafe { I2C_BUS = Some(i2c) };
}
