def add_leds():
    device_db.update({
        "led" + str(i): {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x15+i},
        } for i in range(2)
    })

    return
    device_db.update({
        "led" + str(i+4): {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x000035+i},
        } for i in range(1)
    })

def add_urukul_1():
    device_db.update({
        "spi_urukul0": {
            "type": "local",
            "module": "artiq.coredevice.spi2",
            "class": "SPIMaster",
            "arguments": {"channel": 12}
        },
        "ttl_urukul0_sync": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLClockGen",
            "arguments": {"channel": 13, "acc_width": 4}
        },
        "ttl_urukul0_io_update": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 14}
        },
        "ttl_urukul0_sw0": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 15}
        },
        "ttl_urukul0_sw1": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 16}
        },
        "ttl_urukul0_sw2": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 17}
        },
        "ttl_urukul0_sw3": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 18}
        },
        "urukul0_cpld": {
            "type": "local",
            "module": "artiq.coredevice.urukul",
            "class": "CPLD",
            "arguments": {
                "spi_device": "spi_urukul0",
                "io_update_device": "ttl_urukul0_io_update",
                "sync_device": "ttl_urukul1_sync",
                "refclk": 125e6,
                "clk_sel": 2
            }
        }
    })

    device_db.update({
        "urukul0_ch" + str(i): {
            "type": "local",
            "module": "artiq.coredevice.ad9910",
            "class": "AD9910",
            "arguments": {
                "pll_n": 32,
                "chip_select": 4 + i,
                "cpld_device": "urukul0_cpld",
                "sw_device": "ttl_urukul0_sw" + str(i)
            }
        } for i in range(4)
    })

    device_db.update({
        "ttl_urukul1_sync": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLClockGen",
            "arguments": {"channel": 35, "acc_width": 4}
        },
    })

def add_ttls():
    device_db.update({
        "ttl" + str(i): {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLInOut" if i < 4 else "TTLOut",
            "arguments": {"channel": i},
        } for i in range(24)
    })

def add_urukul():
    device_db.update({
        "spi_urukul0": {
            "type": "local",
            "module": "artiq.coredevice.spi2",
            "class": "SPIMaster",
            "arguments": {"channel": 0x12}
        },
        "ttl_urukul0_io_update": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x13}
        },
        "ttl_urukul0_sw0": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x14}
        },
        "ttl_urukul0_sw1": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x15}
        },
        "ttl_urukul0_sw2": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x16}
        },
        "ttl_urukul0_sw3": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x17}
        },
        "urukul0_cpld": {
            "type": "local",
            "module": "artiq.coredevice.urukul",
            "class": "CPLD",
            "arguments": {
                "spi_device": "spi_urukul0",
                "io_update_device": "ttl_urukul0_io_update",
                "refclk": 125e6,
                "clk_sel": 2,
                "sync_device": None # Possible: "ttl_urukul0_sync",
            }
        }
    })

    device_db.update({
        "urukul0_ch" + str(i): {
            "type": "local",
            "module": "artiq.coredevice.ad9910",
            "class": "AD9910",
            "arguments": {
                "pll_n": 32,
                "chip_select": 4 + i,
                "cpld_device": "urukul0_cpld",
                "sw_device": "ttl_urukul0_sw" + str(i)
            }
        } for i in range(4)
    })

def add_mirny():
    device_db.update({
        "spi_mirny0": {
            "type": "local",
            "module": "artiq.coredevice.spi2",
            "class": "SPIMaster",
            "arguments": {"channel": 0x1e}
        },

        "ttl_mirny0_sw": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x1f}
        },
        "ttl_mirny0_sw1": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x20}
        },
        "ttl_mirny0_sw2": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x21}
        },
        "ttl_mirny0_sw3": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x22}
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
        }
    })

    device_db["mirny0_ch0"] = {
        "type": "local",
        "module": "artiq.coredevice.adf5356",
        "class": "ADF5356",
        "arguments": {
            "channel": 0,
            "sw_device": "ttl_mirny0_sw",  # sw0 değil, sadece sw!
            "cpld_device": "mirny0_cpld"
        }
    }

    # Mirny channels (4 channel, each is ADF5356 PLL)
    for i in range(1,4):
        device_db[f"mirny0_ch{i}"] = {
            "type": "local",
            "module": "artiq.coredevice.adf5356",
            "class": "ADF5356",
            "arguments": {
                "channel": i,
                "sw_device": f"ttl_mirny0_sw{i}",
                "cpld_device": "mirny0_cpld"
            }
        }

ip_host = "192.168.107.67"

device_db = {
    "core": {
        "type": "local",
        "module": "artiq.coredevice.core",
        "class": "Core",
        "arguments": {
            "host": ip_host,
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
}

add_urukul()
add_mirny()