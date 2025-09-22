# Minimal device database with real SPI-based DDS attempt

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

    # Working LED
    "led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 35},  # This is actually Mirny RF2 but works
    },

    # Direct SPI interface (from device mapping)
    "spi_urukul": {
        "type": "local",
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 24}  # From gateware mapping
    },

    # TTL lines for Urukul control
    "urukul_io_update": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 25}
    },
    
    "urukul_sw0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 27}
    },

    # Try creating Urukul CPLD with minimal config
    "urukul_cpld": {
        "type": "local",
        "module": "artiq.coredevice.urukul",
        "class": "CPLD",
        "arguments": {
            "spi_device": "spi_urukul",
            "io_update_device": "urukul_io_update",
            "refclk": 125e6,
            "clk_sel": 2
        }
    },

    # Try AD9910 with minimal config
    "dds0": {
        "type": "local",
        "module": "artiq.coredevice.ad9910",
        "class": "AD9910",
        "arguments": {
            "pll_n": 32,
            "chip_select": 4,
            "cpld_device": "urukul_cpld",
            "sw_device": "urukul_sw0"
        }
    }
}
