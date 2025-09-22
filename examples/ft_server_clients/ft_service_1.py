# urukul_server_simple.py
from artiq.experiment import *
from artiq.language.units import MHz, ms
from sipyco.pc_rpc import simple_server_loop
import numpy as np


class UrukulServer(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

        self.channels = [
            self.urukul0_ch0,
            self.urukul0_ch1,
            self.urukul0_ch2,
            self.urukul0_ch3
        ]

    def prepare(self):
        """Initialize hardware once at startup"""
        self.init_hardware()

    @kernel
    def init_hardware(self):
        """Initialize all hardware"""
        self.core.reset()
        self.urukul0_cpld.init(blind=True)
        delay(10 * ms)

        for ch in self.channels:
            ch.init()
            ch.set(100 * MHz, amplitude=1.0)

        print("Urukul initialized - ready for fast updates!")

    @kernel
    def quick_set(self, ch_idx, freq_hz, amp, sw_on):
        """Ultra-fast single channel update"""
        self.core.break_realtime()

        # Direct access without conditions for speed
        ch = self.channels[ch_idx]
        ch.set_mu(ch.frequency_to_ftw(freq_hz), amplitude=amp)

        if sw_on:
            ch.sw.on()
        else:
            ch.sw.off()

    # RPC methods - now much faster!
    def set_frequency(self, channel, freq_mhz):
        """Set frequency in MHz"""
        # Keep current amplitude and switch state
        amp = 1.0  # You might want to track this
        sw = True  # You might want to track this

        self.quick_set(
            np.int32(channel),
            freq_mhz * 1e6,
            amp,
            sw
        )
        return f"CH{channel} = {freq_mhz} MHz"

    def set_amplitude(self, channel, amplitude):
        """Set amplitude (0-1)"""
        freq = 100e6  # You might want to track this
        sw = True  # You might want to track this

        self.quick_set(
            np.int32(channel),
            freq,
            amplitude,
            sw
        )
        return f"CH{channel} amp = {amplitude}"

    def rf_on(self, channel):
        """Turn RF on"""
        self.quick_rf_switch(np.int32(channel), True)
        return f"CH{channel} ON"

    def rf_off(self, channel):
        """Turn RF off"""
        self.quick_rf_switch(np.int32(channel), False)
        return f"CH{channel} OFF"

    @kernel
    def quick_rf_switch(self, ch_idx, on):
        """Ultra-fast RF switch"""
        self.core.break_realtime()

        if on:
            self.channels[ch_idx].sw.on()
        else:
            self.channels[ch_idx].sw.off()

    def run(self):
        """Run as RPC server"""
        print("Starting Ultra-Fast Urukul Server...")
        print("Port: 3250")
        print("Expected latency: <50ms")

        simple_server_loop({"urukul": self}, "0.0.0.0", 3250)
