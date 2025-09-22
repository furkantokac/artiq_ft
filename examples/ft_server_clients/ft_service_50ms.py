# ft_service.py - WORKING VERSION
from artiq.experiment import *
from artiq.language.units import MHz, ms, us, Hz
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
        # Initialize hardware
        self.init_hardware()

        # Keep track of current settings
        self.current_freq = [100e6, 100e6, 100e6, 100e6]
        self.current_amp = [1.0, 1.0, 1.0, 1.0]
        self.current_sw = [False, False, False, False]
        
        print("Server ready!")

    @kernel
    def init_hardware(self):
        """Initialize hardware once"""
        self.core.reset()
        self.urukul0_cpld.init()
        self.core.break_realtime()
        delay(10*ms)
        
        # Initialize all channels
        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()
        
        self.core.break_realtime()
        
        # Set initial values using full set() method
        self.urukul0_ch0.set(100*MHz, amplitude=1.0)
        self.urukul0_ch1.set(100*MHz, amplitude=1.0)
        self.urukul0_ch2.set(100*MHz, amplitude=1.0)
        self.urukul0_ch3.set(100*MHz, amplitude=1.0)
        
        print("Hardware initialized")

    @kernel
    def update_ch0(self, freq_hz, amp):
        """Update channel 0 - using full set() method"""
        self.core.break_realtime()
        # IMPORTANT: Use set() not set_frequency()!
        self.urukul0_ch0.set(freq_hz*Hz, amplitude=amp)
        
    @kernel
    def update_ch1(self, freq_hz, amp):
        """Update channel 1"""
        self.core.break_realtime()
        self.urukul0_ch1.set(freq_hz*Hz, amplitude=amp)
        
    @kernel
    def update_ch2(self, freq_hz, amp):
        """Update channel 2"""
        self.core.break_realtime()
        self.urukul0_ch2.set(freq_hz*Hz, amplitude=amp)
        
    @kernel
    def update_ch3(self, freq_hz, amp):
        """Update channel 3"""
        self.core.break_realtime()
        self.urukul0_ch3.set(freq_hz*Hz, amplitude=amp)

    @kernel
    def switch_ch0(self, state):
        """Switch channel 0"""
        self.core.break_realtime()
        if state:
            self.urukul0_ch0.sw.on()
        else:
            self.urukul0_ch0.sw.off()
            
    @kernel
    def switch_ch1(self, state):
        """Switch channel 1"""
        self.core.break_realtime()
        if state:
            self.urukul0_ch1.sw.on()
        else:
            self.urukul0_ch1.sw.off()
            
    @kernel
    def switch_ch2(self, state):
        """Switch channel 2"""
        self.core.break_realtime()
        if state:
            self.urukul0_ch2.sw.on()
        else:
            self.urukul0_ch2.sw.off()
            
    @kernel
    def switch_ch3(self, state):
        """Switch channel 3"""
        self.core.break_realtime()
        if state:
            self.urukul0_ch3.sw.on()
        else:
            self.urukul0_ch3.sw.off()

    # RPC methods
    def set_frequency(self, channel, freq_mhz):
        """Set frequency in MHz"""
        freq_hz = freq_mhz * 1e6
        self.current_freq[channel] = freq_hz
        
        # Update hardware with BOTH freq and amp using set()
        if channel == 0:
            self.update_ch0(freq_hz, self.current_amp[0])
        elif channel == 1:
            self.update_ch1(freq_hz, self.current_amp[1])
        elif channel == 2:
            self.update_ch2(freq_hz, self.current_amp[2])
        else:
            self.update_ch3(freq_hz, self.current_amp[3])
            
        print(f"CH{channel} freq = {freq_mhz} MHz")
        return f"CH{channel}={freq_mhz}MHz"

    def set_amplitude(self, channel, amplitude):
        """Set amplitude (0-1)"""
        self.current_amp[channel] = amplitude
        
        # Update hardware with BOTH freq and amp using set()
        if channel == 0:
            self.update_ch0(self.current_freq[0], amplitude)
        elif channel == 1:
            self.update_ch1(self.current_freq[1], amplitude)
        elif channel == 2:
            self.update_ch2(self.current_freq[2], amplitude)
        else:
            self.update_ch3(self.current_freq[3], amplitude)
            
        print(f"CH{channel} amp = {amplitude}")
        return f"CH{channel}={int(amplitude*100)}%"

    def rf_on(self, channel):
        """Turn RF on"""
        self.current_sw[channel] = True
        
        if channel == 0:
            self.switch_ch0(True)
        elif channel == 1:
            self.switch_ch1(True)
        elif channel == 2:
            self.switch_ch2(True)
        else:
            self.switch_ch3(True)
            
        print(f"CH{channel} ON")
        return f"CH{channel} ON"

    def rf_off(self, channel):
        """Turn RF off"""
        self.current_sw[channel] = False
        
        if channel == 0:
            self.switch_ch0(False)
        elif channel == 1:
            self.switch_ch1(False)
        elif channel == 2:
            self.switch_ch2(False)
        else:
            self.switch_ch3(False)
            
        print(f"CH{channel} OFF")
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
        print("Urukul RPC Server - FIXED")
        print("=" * 50)
        print("Port: 3250")
        print("Using set() method for frequency/amplitude updates")
        print("=" * 50)

        simple_server_loop({"urukul": self}, "0.0.0.0", 3250)
