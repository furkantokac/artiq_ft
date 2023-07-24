use libboard_zynq::i2c;
use log::info;

// Only the bare minimum registers. Bits/IO connections equivalent between IC types.
struct Registers {
    // PCA9539 equivalent register names in comments
    iodira: u8, // Configuration Port 0
    iodirb: u8, // Configuration Port 1
    gpioa: u8,  // Output Port 0
    gpiob: u8,  // Output Port 1
}

//IO expanders pins
const IO_DIR_INPUT_ALL: u8 = 0xFF;
const IO_DIR_OUT_SFP_TX_DISABLE: u8 = !0x02;
const IO_DIR_OUT_SFP_LED: u8 = !0x40;
//SFP0 LED has different place in v1.1
#[cfg(hw_rev = "v1.0")]
const IO_DIR_OUT_SFP0_LED: u8 = !0x40;
#[cfg(hw_rev = "v1.1")]
const IO_DIR_OUT_SFP0_LED: u8 = !0x80;

//IO expander port direction
const IO_DIR_MAPPING0: [u8; 2] = [
    IO_DIR_INPUT_ALL & IO_DIR_OUT_SFP_TX_DISABLE & (IO_DIR_OUT_SFP0_LED),
    IO_DIR_INPUT_ALL & IO_DIR_OUT_SFP_TX_DISABLE & IO_DIR_OUT_SFP_LED,
];

const IO_DIR_MAPPING1: [u8; 2] = [
    IO_DIR_INPUT_ALL & IO_DIR_OUT_SFP_TX_DISABLE & IO_DIR_OUT_SFP_LED,
    IO_DIR_INPUT_ALL & IO_DIR_OUT_SFP_TX_DISABLE & IO_DIR_OUT_SFP_LED,
];

pub struct IoExpander<'a> {
    i2c: &'a mut i2c::I2c,
    address: u8,
    iodir: [u8; 2],
    out_current: [u8; 2],
    out_target: [u8; 2],
    registers: Registers,
    virtual_led_mapping: &'static [(u8, u8, u8)],
    channel: usize,
}

impl<'a> IoExpander<'a> {
    pub fn new(i2c: &'a mut i2c::I2c, index: u8) -> Result<Self, &'static str> {
        #[cfg(hw_rev = "v1.0")]
        const VIRTUAL_LED_MAPPING0: [(u8, u8, u8); 2] = [(0, 0, 6), (1, 1, 6)];
        #[cfg(hw_rev = "v1.1")]
        const VIRTUAL_LED_MAPPING0: [(u8, u8, u8); 2] = [(0, 0, 7), (1, 1, 6)];

        const VIRTUAL_LED_MAPPING1: [(u8, u8, u8); 2] = [(2, 0, 6), (3, 1, 6)];

        // Both expanders on SHARED I2C bus
        let mut io_expander = match index {
            0 => IoExpander {
                i2c,
                address: 0x40,
                iodir: IO_DIR_MAPPING0,
                out_current: [0; 2],
                out_target: [0; 2],
                registers: Registers {
                    iodira: 0x00,
                    iodirb: 0x01,
                    gpioa: 0x12,
                    gpiob: 0x13,
                },
                virtual_led_mapping: &VIRTUAL_LED_MAPPING0,
                channel: 0,
            },
            1 => IoExpander {
                i2c,
                address: 0x42,
                iodir: IO_DIR_MAPPING1,
                out_current: [0; 2],
                out_target: [0; 2],
                registers: Registers {
                    iodira: 0x00,
                    iodirb: 0x01,
                    gpioa: 0x12,
                    gpiob: 0x13,
                },
                virtual_led_mapping: &VIRTUAL_LED_MAPPING1,
                channel: 1,
            },
            _ => return Err("incorrect I/O expander index"),
        };
        if !io_expander.check_ack()? {
            info!("MCP23017 io expander {} not found. Checking for PCA9539.", index);
            io_expander.address += 0xa8; // translate to PCA9539 addresses (see schematic)
            io_expander.registers = Registers {
                iodira: 0x06,
                iodirb: 0x07,
                gpioa: 0x02,
                gpiob: 0x03,
            };
            if !io_expander.check_ack()? {
                return Err("Neither MCP23017 nor PCA9539 io expander found.");
            };
        }
        Ok(io_expander)
    }

    fn select(&mut self) -> Result<(), &'static str> {
        self.i2c.pca954x_select(0x70, None)?;
        self.i2c.pca954x_select(0x71, Some(3))?;
        Ok(())
    }

    fn write(&mut self, addr: u8, value: u8) -> Result<(), &'static str> {
        self.i2c.start()?;
        self.i2c.write(self.address)?;
        self.i2c.write(addr)?;
        self.i2c.write(value)?;
        self.i2c.stop()?;
        Ok(())
    }

    fn check_ack(&mut self) -> Result<bool, &'static str> {
        // Check for ack from io expander
        self.select()?;
        self.i2c.start()?;
        let ack = self.i2c.write(self.address)?;
        self.i2c.stop()?;
        Ok(ack)
    }

    fn update_iodir(&mut self) -> Result<(), &'static str> {
        self.write(self.registers.iodira, self.iodir[0])?;
        self.write(self.registers.iodirb, self.iodir[1])?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        self.select()?;

        self.iodir = match self.channel {
            0 => IO_DIR_MAPPING0,
            1 => IO_DIR_MAPPING1,
            _ => [IO_DIR_INPUT_ALL; 2],
        };

        self.update_iodir()?;

        self.out_current[0] = 0x00;
        self.write(self.registers.gpioa, 0x00)?;
        self.out_current[1] = 0x00;
        self.write(self.registers.gpiob, 0x00)?;
        Ok(())
    }

    pub fn set_oe(&mut self, port: u8, outputs: u8) -> Result<(), &'static str> {
        self.iodir[port as usize] &= !outputs;
        self.update_iodir()?;
        Ok(())
    }

    pub fn set(&mut self, port: u8, bit: u8, high: bool) {
        if high {
            self.out_target[port as usize] |= 1 << bit;
        } else {
            self.out_target[port as usize] &= !(1 << bit);
        }
    }

    pub fn led_update(&mut self, led_state: u8) {
        for (led_num, port, bit) in self.virtual_led_mapping.iter() {
            let level = (led_state >> led_num) & 1;
            self.set(*port, *bit, level != 0);
        }
        self.service().unwrap();
    }

    pub fn service(&mut self) -> Result<(), &'static str> {
        if self.out_target != self.out_current {
            self.select()?;
            if self.out_target[0] != self.out_current[0] {
                self.write(self.registers.gpioa, self.out_target[0])?;
                self.out_current[0] = self.out_target[0];
            }
            if self.out_target[1] != self.out_current[1] {
                self.write(self.registers.gpiob, self.out_target[1])?;
                self.out_current[1] = self.out_target[1];
            }
        }

        Ok(())
    }
}
