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

    # SPI bus for Mirny
    "spi_mirny0": {
        "type": "local",
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 32}
    },

    # Mirny device
    "mirny0": {
        "type": "local",
        "module": "artiq.coredevice.mirny",
        "class": "Mirny",
        "arguments": {
            "spi_device": "spi_mirny0",
            "refclk": 100e6,  # 100 MHz reference clock
            "clk_sel": "XO"  # XO, SMA, veya MMCX seçenekleri
        }
    },

    # Urukul (AD9910) - based on correct_device_db.py
    "spi_urukul0": {
        "type": "local",
        "module": "artiq.coredevice.spi2",
        "class": "SPIMaster",
        "arguments": {"channel": 24}  # From correct_device_db mapping
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
        "class": "TTLClockGen",
        "arguments": {"channel": 25}
    },
    
    "ttl_urukul0_io_update": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 26}
    },

    # Urukul RF switches
    "ttl_urukul0_sw0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 27}
    },

    # AD9910 DDS channels
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

    # USER LEDs - demo.json'dan çıkarılan doğru channel'lar
    "led0": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 50},  # USER LED 0
    },
    "led1": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 51},  # USER LED 1
    },
    "led2": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 52},  # USER LED 1
    },
    "led3": {
        "type": "local",
        "module": "artiq.coredevice.ttl",
        "class": "TTLOut",
        "arguments": {"channel": 53},  # USER LED 1
    },
}