# For NIST_QC2

device_db = {
    "core": {
        "type": "local",
        "module": "artiq.coredevice.core",
        "class": "Core",
        "arguments": {
            "host": "192.168.1.52",
            "ref_period": 1e-9,
            "ref_multiplier": 1,
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

    "i2c_switch": {
        "type": "local",
        "module": "artiq.coredevice.i2c",
        "class": "PCA9548"
    },

    # led? are common to all variants
    "led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 0},
    },
    "led1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 1},
    },
    "led2": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 2}
    },
    "led3": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 3}
    },
}

# TTLs on QC2 backplane
for i in range(40):
    device_db["ttl" + str(i)] = {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLInOut",
        "arguments": {"channel": 4+i}
    }

device_db["ad9914dds0"] = {
    "type": "local",
    "module": "artiq.coredevice.ad9914",
    "class": "AD9914",
    "arguments": {"sysclk": 3e9, "bus_channel": 50, "channel": 0},
}
device_db["ad9914dds1"] = {
    "type": "local",
    "module": "artiq.coredevice.ad9914",
    "class": "AD9914",
    "arguments": {"sysclk": 3e9, "bus_channel": 50, "channel": 1},
}

# for ARTIQ test suite
device_db.update(
    loop_out="ttl0",
    loop_in="ttl1",
    ttl_out="ttl2",
    ttl_out_serdes="ttl2",
)
