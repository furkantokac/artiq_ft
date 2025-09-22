# urukul_server.py
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

    def prepare(self):
        """Prepare experiment"""
        # Initialize hardware once
        self.init_hardware()

        # Keep track of current settings
        self.current_freq = [100e6, 100e6, 100e6, 100e6]
        self.current_amp = [1.0, 1.0, 1.0, 1.0]
        self.current_sw = [False, False, False, False]

    @kernel
    def init_hardware(self):
        """Initialize hardware once"""
        self.core.reset()
        self.urukul0_cpld.init(blind=True)
        delay(10 * ms)

        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()

        # Set initial values
        self.urukul0_ch0.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch1.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch2.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch3.set(100 * MHz, amplitude=1.0)

        print("Urukul initialized!")

    @kernel
    def fast_freq_ch0(self, freq_hz):
        """Ultra-fast frequency update for ch0"""
        self.core.break_realtime()
        self.urukul0_ch0.set_frequency(freq_hz)

    @kernel
    def fast_freq_ch1(self, freq_hz):
        """Ultra-fast frequency update for ch1"""
        self.core.break_realtime()
        self.urukul0_ch1.set_frequency(freq_hz)

    @kernel
    def fast_freq_ch2(self, freq_hz):
        """Ultra-fast frequency update for ch2"""
        self.core.break_realtime()
        self.urukul0_ch2.set_frequency(freq_hz)

    @kernel
    def fast_freq_ch3(self, freq_hz):
        """Ultra-fast frequency update for ch3"""
        self.core.break_realtime()
        self.urukul0_ch3.set_frequency(freq_hz)

    @kernel
    def fast_amp_ch0(self, amp):
        """Ultra-fast amplitude update for ch0"""
        self.core.break_realtime()
        self.urukul0_ch0.set_amplitude(amp)

    @kernel
    def fast_amp_ch1(self, amp):
        """Ultra-fast amplitude update for ch1"""
        self.core.break_realtime()
        self.urukul0_ch1.set_amplitude(amp)

    @kernel
    def fast_amp_ch2(self, amp):
        """Ultra-fast amplitude update for ch2"""
        self.core.break_realtime()
        self.urukul0_ch2.set_amplitude(amp)

    @kernel
    def fast_amp_ch3(self, amp):
        """Ultra-fast amplitude update for ch3"""
        self.core.break_realtime()
        self.urukul0_ch3.set_amplitude(amp)

    @kernel
    def fast_sw_on_ch0(self):
        """Ultra-fast switch ON for ch0"""
        self.core.break_realtime()
        self.urukul0_ch0.sw.on()

    @kernel
    def fast_sw_off_ch0(self):
        """Ultra-fast switch OFF for ch0"""
        self.core.break_realtime()
        self.urukul0_ch0.sw.off()

    @kernel
    def fast_sw_on_ch1(self):
        """Ultra-fast switch ON for ch1"""
        self.core.break_realtime()
        self.urukul0_ch1.sw.on()

    @kernel
    def fast_sw_off_ch1(self):
        """Ultra-fast switch OFF for ch1"""
        self.core.break_realtime()
        self.urukul0_ch1.sw.off()

    @kernel
    def fast_sw_on_ch2(self):
        """Ultra-fast switch ON for ch2"""
        self.core.break_realtime()
        self.urukul0_ch2.sw.on()

    @kernel
    def fast_sw_off_ch2(self):
        """Ultra-fast switch OFF for ch2"""
        self.core.break_realtime()
        self.urukul0_ch2.sw.off()

    @kernel
    def fast_sw_on_ch3(self):
        """Ultra-fast switch ON for ch3"""
        self.core.break_realtime()
        self.urukul0_ch3.sw.on()

    @kernel
    def fast_sw_off_ch3(self):
        """Ultra-fast switch OFF for ch3"""
        self.core.break_realtime()
        self.urukul0_ch3.sw.off()

    # RPC methods for GUI - optimized for speed
    def set_frequency(self, channel, freq_mhz):
        """Set frequency in MHz - ULTRA FAST"""
        freq_hz = freq_mhz * 1e6
        self.current_freq[channel] = freq_hz

        # Direct call to specific kernel - no conditionals
        if channel == 0:
            self.fast_freq_ch0(freq_hz)
        elif channel == 1:
            self.fast_freq_ch1(freq_hz)
        elif channel == 2:
            self.fast_freq_ch2(freq_hz)
        else:
            self.fast_freq_ch3(freq_hz)

        return f"CH{channel} = {freq_mhz} MHz"

    def set_amplitude(self, channel, amplitude):
        """Set amplitude (0-1) - ULTRA FAST"""
        self.current_amp[channel] = amplitude

        # Direct call to specific kernel
        if channel == 0:
            self.fast_amp_ch0(amplitude)
        elif channel == 1:
            self.fast_amp_ch1(amplitude)
        elif channel == 2:
            self.fast_amp_ch2(amplitude)
        else:
            self.fast_amp_ch3(amplitude)

        return f"CH{channel} amp = {amplitude}"

    def rf_on(self, channel):
        """Turn RF on - ULTRA FAST"""
        self.current_sw[channel] = True

        # Direct call to specific kernel
        if channel == 0:
            self.fast_sw_on_ch0()
        elif channel == 1:
            self.fast_sw_on_ch1()
        elif channel == 2:
            self.fast_sw_on_ch2()
        else:
            self.fast_sw_on_ch3()

        return f"CH{channel} ON"

    def rf_off(self, channel):
        """Turn RF off - ULTRA FAST"""
        self.current_sw[channel] = False

        # Direct call to specific kernel
        if channel == 0:
            self.fast_sw_off_ch0()
        elif channel == 1:
            self.fast_sw_off_ch1()
        elif channel == 2:
            self.fast_sw_off_ch2()
        else:
            self.fast_sw_off_ch3()

        return f"CH{channel} OFF"

    def get_state(self, channel=None):
        """Query current state"""
        if channel is not None:
            return {
                "freq_mhz": self.current_freq[channel] / 1e6,
                "amplitude": self.current_amp[channel],
                "on": self.current_sw[channel]
            }
        return {
            i: {
                "freq_mhz": self.current_freq[i] / 1e6,
                "amplitude": self.current_amp[i],
                "on": self.current_sw[i]
            } for i in range(4)
        }

    def run(self):
        """Run as RPC server"""
        print("=" * 50)
        print("Ultra-Fast Urukul RPC Server")
        print("=" * 50)
        print("Port: 3250")
        print("Optimizations:")
        print(" - Minimal kernel size (single operation)")
        print(" - No conditionals in kernels")
        print(" - Direct channel-specific kernels")
        print(" - Expected latency: <10-20ms")
        print("=" * 50)

        # Start RPC server
        simple_server_loop({"urukul": self}, "0.0.0.0", 3250)
