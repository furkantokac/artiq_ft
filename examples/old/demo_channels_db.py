# Device database based on ACTUAL demo.json gateware channel mapping

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

    # Real user LEDs from demo.json output
    "user_led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 52},  # 0x000034 = 52
    },
    "user_led1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 53},  # 0x000035 = 53
    },

    # Try using the channels that demo.json would create
    # Urukul starts at channel 0x000012 = 18
    "urukul_test": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 18},  # First Urukul channel
    },

    # Mirny starts at channel 0x00001e = 30
    "mirny_test": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 30},  # First Mirny channel
    },

    # Use current working LED for compatibility
    "led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 35},  # This works (Mirny RF2)
    }
}
