# Device database with working DDS channels discovered

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
        "arguments": {"channel": 35},
    },

    # Real working DDS channels - try AD9910 configuration
    "urukul_dds": {
        "type": "local",
        "module": "artiq.coredevice.ad9910",
        "class": "AD9910",
        "arguments": {
            "pll_n": 32,
            "chip_select": 18,  # Use the working channel directly
            "sw_device": "urukul_sw"
        }
    },

    "urukul_sw": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 19}  # Next channel for switch
    },

    # Try Mirny PLL
    "mirny_pll": {
        "type": "local",
        "module": "artiq.coredevice.adf5356",
        "class": "ADF5356",
        "arguments": {
            "channel": 30,  # Use working channel
            "sw_device": "mirny_sw"
        }
    },

    "mirny_sw": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 31}  # Next channel for switch
    }
}
