# Kasli-SoC Device Database with Urukul and Mirny modules

device_db = {
    "core": {
        "type": "local",
        "module": "artiq.coredevice.core",
        "class": "Core",
        "arguments": {
            "host": "192.168.107.67",
            "ref_period": 1e-9,
            "ref_multiplier": 8,
            "target": "cortexa9"
        }
    },
    "core_cache": {
        "type": "local",
        "module": "artiq.coredevice.cache",
        "class": "CoreCache"
    },
    "core_dma": {
        "type": "local",
        "module": "artiq.coredevice.dma",
        "class": "CoreDMA"
    },

    # LEDs
    "led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 52},  # USER LED 0
    },
    "led1": {
        "type": "local",
        "module": "artiq.coredevice.ttl", 
        "class": "TTLOut",
        "arguments": {"channel": 53},  # USER LED 1
    },

    # Urukul (AD9910 DDS) - EEM ports 3-4
    "urukul0_cpld": {
        "type": "local",
        "module": "artiq.coredevice.urukul",
        "class": "CPLD",
        "arguments": {
            "spi_device": "spi_urukul0",
            "sync_device": "ttl_urukul0_sync",
            "io_update_device": "ttl_urukul0_io_update",
            "refclk": 125e6,
            "clk_sel": 2
        }
    },
    "spi_urukul0": {
        "type": "local",
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 24}  # Based on EEM3-4 mapping
    },
    "ttl_urukul0_sync": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 25}
    },
    "ttl_urukul0_io_update": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 26}
    },

    # Urukul DDS channels
    "urukul0_ch0": {
        "type": "local",
        "module": "artiq.coredevice.ad9910",
        "class": "AD9910",
        "arguments": {
            "pll_n": 32,
            "chip_select": 4,
            "cpld_device": "urukul0_cpld",
            "sw_device": "ttl_urukul0_sw0"
        }
    },
    "urukul0_ch1": {
        "type": "local",
        "module": "artiq.coredevice.ad9910", 
        "class": "AD9910",
        "arguments": {
            "pll_n": 32,
            "chip_select": 5,
            "cpld_device": "urukul0_cpld",
            "sw_device": "ttl_urukul0_sw1"
        }
    },
    "urukul0_ch2": {
        "type": "local",
        "module": "artiq.coredevice.ad9910",
        "class": "AD9910", 
        "arguments": {
            "pll_n": 32,
            "chip_select": 6,
            "cpld_device": "urukul0_cpld",
            "sw_device": "ttl_urukul0_sw2"
        }
    },
    "urukul0_ch3": {
        "type": "local",
        "module": "artiq.coredevice.ad9910",
        "class": "AD9910",
        "arguments": {
            "pll_n": 32,
            "chip_select": 7,
            "cpld_device": "urukul0_cpld", 
            "sw_device": "ttl_urukul0_sw3"
        }
    },

    # Urukul RF switches
    "ttl_urukul0_sw0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 27}
    },
    "ttl_urukul0_sw1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut", 
        "arguments": {"channel": 28}
    },
    "ttl_urukul0_sw2": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 29}
    },
    "ttl_urukul0_sw3": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 30}
    },

    # Mirny (Quad PLL) - EEM port 8
    "mirny0_cpld": {
        "type": "local",
        "module": "artiq.coredevice.mirny",
        "class": "Mirny",
        "arguments": {
            "spi_device": "spi_mirny0",
            "refclk": 125e6,
            "clk_sel": 1
        }
    },
    "spi_mirny0": {
        "type": "local", 
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 32}  # Based on EEM8 mapping
    },

    # Mirny PLL channels - Correct ADF5356 configuration
    "mirny0_ch0": {
        "type": "local",
        "module": "artiq.coredevice.adf5356",
        "class": "ADF5356",
        "arguments": {
            "channel": 0,
            "cpld_device": "mirny0_cpld"
        }
    },
    "mirny0_ch1": {
        "type": "local",
        "module": "artiq.coredevice.adf5356",
        "class": "ADF5356", 
        "arguments": {
            "channel": 1,
            "cpld_device": "mirny0_cpld"
        }
    },
    "mirny0_ch2": {
        "type": "local",
        "module": "artiq.coredevice.adf5356",
        "class": "ADF5356",
        "arguments": {
            "channel": 2,
            "cpld_device": "mirny0_cpld"
        }
    },
    "mirny0_ch3": {
        "type": "local",
        "module": "artiq.coredevice.adf5356", 
        "class": "ADF5356",
        "arguments": {
            "channel": 3,
            "cpld_device": "mirny0_cpld"
        }
    }
}
