def add_leds():
    """
    device_db.update({
        "led0": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 2},
        },
        "led1": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 3},
        },
        "led2": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 4},
        },
        "led3": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 5},
        },

        "led4": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 7},
        },
        "led5": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 8},
        },
        "led6": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 9},
        },
        "led7": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 10},
        },
    })
    """

    device_db.update({
        "led" + str(i): {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0x12+i},
        } for i in range(4)
    })

    device_db.update({
        "led" + str(i+4): {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLOut",
            "arguments": {"channel": 0xe+i},
        } for i in range(4)
    })

def add_urukul1():
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

def add_urukul2():
    device_db.update({
        # --- RTIO low-level devices (kanal numaraları kasli_soc.py çıktısından) ---
        # Urukul base at 0x000012
        "spi_urukul0": {
            "type": "local",
            "module": "artiq.coredevice.spi2",
            "class": "SPI",
            "arguments": {"channel": 0x12},  # 18
        },
        "ttl_urukul0_io_update": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLInOut",
            "arguments": {"channel": 0x13},  # 19
        },
        "ttl_urukul0_sw0": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLInOut",
            "arguments": {"channel": 0x14},  # 20
        },
        "ttl_urukul0_sw1": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLInOut",
            "arguments": {"channel": 0x15},  # 21
        },
        "ttl_urukul0_sw2": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLInOut",
            "arguments": {"channel": 0x16},  # 22
        },
        "ttl_urukul0_sw3": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLInOut",
            "arguments": {"channel": 0x17},  # 23
        },

        # --- Urukul high-level ---
        "urukul0_cpld": {
            "type": "local",
            "module": "artiq.devices.urukul",
            "class": "CPLD",
            "arguments": {
                "spi_device": "spi_urukul0",
                "io_update": "ttl_urukul0_io_update",
                # If you have sync/reset connections, add below
                # "sync": "ttl_urukul0_sync",
                # "reset": "ttl_urukul0_reset",
            },
        },

        "urukul0_ch0": {
            "type": "local",
            "module": "artiq.devices.ad9910",
            "class": "AD9910",
            "arguments": {"cpld": "urukul0_cpld", "chip_select": 0, "sw": "ttl_urukul0_sw0"},
        },
        "urukul0_ch1": {
            "type": "local",
            "module": "artiq.devices.ad9910",
            "class": "AD9910",
            "arguments": {"cpld": "urukul0_cpld", "chip_select": 1, "sw": "ttl_urukul0_sw1"},
        },
        "urukul0_ch2": {
            "type": "local",
            "module": "artiq.devices.ad9910",
            "class": "AD9910",
            "arguments": {"cpld": "urukul0_cpld", "chip_select": 2, "sw": "ttl_urukul0_sw2"},
        },
        "urukul0_ch3": {
            "type": "local",
            "module": "artiq.devices.ad9910",
            "class": "AD9910",
            "arguments": {"cpld": "urukul0_cpld", "chip_select": 3, "sw": "ttl_urukul0_sw3"},
        },
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
                "sync_device": None
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

    '''
    device_db.update({
        "ttl_urukul1_sync": {
            "type": "local",
            "module": "artiq.coredevice.ttl",
            "class": "TTLClockGen",
            "arguments": {"channel": 35, "acc_width": 4}
        },
    })
    '''

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