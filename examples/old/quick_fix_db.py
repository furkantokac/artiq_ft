# Quick fix device database - minimal but working configuration

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

    # Direct DDS channels - bypass CPLD completely
    # Use the working channels as direct DDS
    "dds0": {
        "type": "local",
        "module": "artiq.coredevice.ad9914",  # Back to AD9914 since it was working
        "class": "AD9914",
        "arguments": {
            "sysclk": 3e9, 
            "bus_channel": 18,  # Use working channel 18
            "channel": 0
        }
    },

    "dds1": {
        "type": "local",
        "module": "artiq.coredevice.ad9914",
        "class": "AD9914", 
        "arguments": {
            "sysclk": 3e9,
            "bus_channel": 30,  # Use working channel 30
            "channel": 1
        }
    },

    # TTL switches for RF control
    "rf_switch0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 19}  # Next to channel 18
    },

    "rf_switch1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 31}  # Next to channel 30
    }
}
