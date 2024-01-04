use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;
use libboard_zynq::timer::GlobalTimer;
use log::info;

use crate::pl::csr;

#[cfg(feature = "target_kasli_soc")]
const ADDRESS: u8 = 0x67;

const ADPLL_MAX: i32 = (950.0 / 0.0001164) as i32;

pub struct DividerConfig {
    pub hsdiv: u16,
    pub lsdiv: u8,
    pub fbdiv: u64,
}

pub struct FrequencySetting {
    pub main: DividerConfig,
    pub helper: DividerConfig,
}

mod i2c {
    use super::*;

    #[derive(Clone, Copy)]
    pub enum DCXO {
        Main,
        Helper,
    }

    fn half_period(timer: &mut GlobalTimer) {
        timer.delay_us(1)
    }

    fn sda_i(dcxo: DCXO) -> bool {
        match dcxo {
            DCXO::Main => unsafe { csr::wrpll::main_dcxo_sda_in_read() == 1 },
            DCXO::Helper => unsafe { csr::wrpll::helper_dcxo_sda_in_read() == 1 },
        }
    }

    fn sda_oe(dcxo: DCXO, oe: bool) {
        let val = if oe { 1 } else { 0 };
        match dcxo {
            DCXO::Main => unsafe { csr::wrpll::main_dcxo_sda_oe_write(val) },
            DCXO::Helper => unsafe { csr::wrpll::helper_dcxo_sda_oe_write(val) },
        };
    }

    fn sda_o(dcxo: DCXO, o: bool) {
        let val = if o { 1 } else { 0 };
        match dcxo {
            DCXO::Main => unsafe { csr::wrpll::main_dcxo_sda_out_write(val) },
            DCXO::Helper => unsafe { csr::wrpll::helper_dcxo_sda_out_write(val) },
        };
    }

    fn scl_oe(dcxo: DCXO, oe: bool) {
        let val = if oe { 1 } else { 0 };
        match dcxo {
            DCXO::Main => unsafe { csr::wrpll::main_dcxo_scl_oe_write(val) },
            DCXO::Helper => unsafe { csr::wrpll::helper_dcxo_scl_oe_write(val) },
        };
    }

    fn scl_o(dcxo: DCXO, o: bool) {
        let val = if o { 1 } else { 0 };
        match dcxo {
            DCXO::Main => unsafe { csr::wrpll::main_dcxo_scl_out_write(val) },
            DCXO::Helper => unsafe { csr::wrpll::helper_dcxo_scl_out_write(val) },
        };
    }

    pub fn init(dcxo: DCXO, timer: &mut GlobalTimer) -> Result<(), &'static str> {
        // Set SCL as output, and high level
        scl_o(dcxo, true);
        scl_oe(dcxo, true);
        // Prepare a zero level on SDA so that sda_oe pulls it down
        sda_o(dcxo, false);
        // Release SDA
        sda_oe(dcxo, false);

        // Check the I2C bus is ready
        half_period(timer);
        half_period(timer);
        if !sda_i(dcxo) {
            // Try toggling SCL a few times
            for _bit in 0..8 {
                scl_o(dcxo, false);
                half_period(timer);
                scl_o(dcxo, true);
                half_period(timer);
            }
        }

        if !sda_i(dcxo) {
            return Err("SDA is stuck low and doesn't get unstuck");
        }
        Ok(())
    }

    pub fn start(dcxo: DCXO, timer: &mut GlobalTimer) {
        // Set SCL high then SDA low
        scl_o(dcxo, true);
        half_period(timer);
        sda_oe(dcxo, true);
        half_period(timer);
    }

    pub fn stop(dcxo: DCXO, timer: &mut GlobalTimer) {
        // First, make sure SCL is low, so that the target releases the SDA line
        scl_o(dcxo, false);
        half_period(timer);
        // Set SCL high then SDA high
        sda_oe(dcxo, true);
        scl_o(dcxo, true);
        half_period(timer);
        sda_oe(dcxo, false);
        half_period(timer);
    }

    pub fn write(dcxo: DCXO, data: u8, timer: &mut GlobalTimer) -> bool {
        // MSB first
        for bit in (0..8).rev() {
            // Set SCL low and set our bit on SDA
            scl_o(dcxo, false);
            sda_oe(dcxo, data & (1 << bit) == 0);
            half_period(timer);
            // Set SCL high ; data is shifted on the rising edge of SCL
            scl_o(dcxo, true);
            half_period(timer);
        }
        // Check ack
        // Set SCL low, then release SDA so that the I2C target can respond
        scl_o(dcxo, false);
        half_period(timer);
        sda_oe(dcxo, false);
        // Set SCL high and check for ack
        scl_o(dcxo, true);
        half_period(timer);
        // returns true if acked (I2C target pulled SDA low)
        !sda_i(dcxo)
    }

    pub fn read(dcxo: DCXO, ack: bool, timer: &mut GlobalTimer) -> u8 {
        // Set SCL low first, otherwise setting SDA as input may cause a transition
        // on SDA with SCL high which will be interpreted as START/STOP condition.
        scl_o(dcxo, false);
        half_period(timer); // make sure SCL has settled low
        sda_oe(dcxo, false);

        let mut data: u8 = 0;

        // MSB first
        for bit in (0..8).rev() {
            scl_o(dcxo, false);
            half_period(timer);
            // Set SCL high and shift data
            scl_o(dcxo, true);
            half_period(timer);
            if sda_i(dcxo) {
                data |= 1 << bit
            }
        }
        // Send ack
        // Set SCL low and pull SDA low when acking
        scl_o(dcxo, false);
        if ack {
            sda_oe(dcxo, true)
        }
        half_period(timer);
        // then set SCL high
        scl_o(dcxo, true);
        half_period(timer);

        data
    }
}

fn write(dcxo: i2c::DCXO, reg: u8, val: u8, timer: &mut GlobalTimer) -> Result<(), &'static str> {
    i2c::start(dcxo, timer);
    if !i2c::write(dcxo, ADDRESS << 1, timer) {
        return Err("Si549 failed to ack write address");
    }
    if !i2c::write(dcxo, reg, timer) {
        return Err("Si549 failed to ack register");
    }
    if !i2c::write(dcxo, val, timer) {
        return Err("Si549 failed to ack value");
    }
    i2c::stop(dcxo, timer);
    Ok(())
}

fn read(dcxo: i2c::DCXO, reg: u8, timer: &mut GlobalTimer) -> Result<u8, &'static str> {
    i2c::start(dcxo, timer);
    if !i2c::write(dcxo, ADDRESS << 1, timer) {
        return Err("Si549 failed to ack write address");
    }
    if !i2c::write(dcxo, reg, timer) {
        return Err("Si549 failed to ack register");
    }
    i2c::stop(dcxo, timer);

    i2c::start(dcxo, timer);
    if !i2c::write(dcxo, (ADDRESS << 1) | 1, timer) {
        return Err("Si549 failed to ack read address");
    }
    let val = i2c::read(dcxo, false, timer);
    i2c::stop(dcxo, timer);
    Ok(val)
}

fn setup(dcxo: i2c::DCXO, config: &DividerConfig, timer: &mut GlobalTimer) -> Result<(), &'static str> {
    i2c::init(dcxo, timer)?;

    write(dcxo, 255, 0x00, timer)?; // PAGE
    write(dcxo, 69, 0x00, timer)?; // Disable FCAL override.
    write(dcxo, 17, 0x00, timer)?; // Synchronously disable output

    // The Si549 has no ID register, so we check that it responds correctly
    // by writing values to a RAM-like register and reading them back.
    for test_value in 0..255 {
        write(dcxo, 23, test_value, timer)?;
        let readback = read(dcxo, 23, timer)?;
        if readback != test_value {
            return Err("Si549 detection failed");
        }
    }

    write(dcxo, 23, config.hsdiv as u8, timer)?;
    write(dcxo, 24, (config.hsdiv >> 8) as u8 | (config.lsdiv << 4), timer)?;
    write(dcxo, 26, config.fbdiv as u8, timer)?;
    write(dcxo, 27, (config.fbdiv >> 8) as u8, timer)?;
    write(dcxo, 28, (config.fbdiv >> 16) as u8, timer)?;
    write(dcxo, 29, (config.fbdiv >> 24) as u8, timer)?;
    write(dcxo, 30, (config.fbdiv >> 32) as u8, timer)?;
    write(dcxo, 31, (config.fbdiv >> 40) as u8, timer)?;

    write(dcxo, 7, 0x08, timer)?; // Start FCAL
    timer.delay_us(30_000); // Internal FCAL VCO calibration
    write(dcxo, 17, 0x01, timer)?; // Synchronously enable output

    Ok(())
}

pub fn main_setup(timer: &mut GlobalTimer, settings: &FrequencySetting) -> Result<(), &'static str> {
    unsafe {
        csr::wrpll::main_dcxo_bitbang_enable_write(1);
        csr::wrpll::main_dcxo_i2c_address_write(ADDRESS);
    }

    setup(i2c::DCXO::Main, &settings.main, timer)?;

    // Si549 maximum settling time for large frequency change.
    timer.delay_us(40_000);

    unsafe {
        csr::wrpll::main_dcxo_bitbang_enable_write(0);
    }

    info!("Main Si549 started");
    Ok(())
}

pub fn helper_setup(timer: &mut GlobalTimer, settings: &FrequencySetting) -> Result<(), &'static str> {
    unsafe {
        csr::wrpll::helper_reset_write(1);
        csr::wrpll::helper_dcxo_bitbang_enable_write(1);
        csr::wrpll::helper_dcxo_i2c_address_write(ADDRESS);
    }

    setup(i2c::DCXO::Helper, &settings.helper, timer)?;

    // Si549 maximum settling time for large frequency change.
    timer.delay_us(40_000);

    unsafe {
        csr::wrpll::helper_reset_write(0);
        csr::wrpll::helper_dcxo_bitbang_enable_write(0);
    }
    info!("Helper Si549 started");
    Ok(())
}

fn set_adpll(dcxo: i2c::DCXO, adpll: i32) -> Result<(), &'static str> {
    if adpll.abs() > ADPLL_MAX {
        return Err("adpll is too large");
    }

    match dcxo {
        i2c::DCXO::Main => unsafe {
            if csr::wrpll::main_dcxo_bitbang_enable_read() == 1 {
                return Err("Main si549 bitbang mode is active when using gateware i2c");
            }

            while csr::wrpll::main_dcxo_adpll_busy_read() == 1 {}
            if csr::wrpll::main_dcxo_nack_read() == 1 {
                return Err("Main si549 failed to ack adpll write");
            }

            csr::wrpll::main_dcxo_i2c_address_write(ADDRESS);
            csr::wrpll::main_dcxo_adpll_write(adpll as u32);

            csr::wrpll::main_dcxo_adpll_stb_write(1);
        },
        i2c::DCXO::Helper => unsafe {
            if csr::wrpll::helper_dcxo_bitbang_enable_read() == 1 {
                return Err("Helper si549 bitbang mode is active when using gateware i2c");
            }

            while csr::wrpll::helper_dcxo_adpll_busy_read() == 1 {}
            if csr::wrpll::helper_dcxo_nack_read() == 1 {
                return Err("Helper si549 failed to ack adpll write");
            }

            csr::wrpll::helper_dcxo_i2c_address_write(ADDRESS);
            csr::wrpll::helper_dcxo_adpll_write(adpll as u32);

            csr::wrpll::helper_dcxo_adpll_stb_write(1);
        },
    };

    Ok(())
}
