# Correct Device Database for actual hardware: AD9910 Urukul + Mirny

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

    # Real User LEDs (from device mapping)
    "user_led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 52},  # Actual user LED
    },
    "user_led1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 53},  # Actual user LED
    },

    # Urukul (AD9910) - based on device mapping
    "spi_urukul0": {
        "type": "local",
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 24}  # From device map
    },
    
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
    
    "ttl_urukul0_sync": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLInOut",
        "arguments": {"channel": 25}
    },
    
    "ttl_urukul0_io_update": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 26}
    },

    # Urukul RF switches (from device map)
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

    # AD9910 DDS channels (correct chip for Urukul)
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

    # Mirny (from device mapping)
    "spi_mirny0": {
        "type": "local",
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 32}
    },
    
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

    # Mirny RF switches
    "ttl_mirny0_sw": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 33}
    },
    "ttl_mirny0_sw1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 34}
    },
    "ttl_mirny0_sw2": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 35}  # This was your "led0"!
    },
    "ttl_mirny0_sw3": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 36}
    }
}
