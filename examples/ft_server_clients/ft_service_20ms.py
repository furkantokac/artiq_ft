# ft_service_force_update.py
from artiq.experiment import *
from artiq.language.units import MHz, ms, us
from sipyco.pc_rpc import simple_server_loop


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
        self.init_hardware()
        
        # Current settings
        self.current_freq = [100e6, 100e6, 100e6, 100e6]
        self.current_amp = [1.0, 1.0, 1.0, 1.0]
        self.current_sw = [False, False, False, False]

    @kernel
    def init_hardware(self):
        """Initialize hardware"""
        self.core.reset()
        self.urukul0_cpld.init()
        delay(10*ms)
        
        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()
        
        # Initial setup
        self.urukul0_ch0.set(100*MHz, amplitude=1.0)
        self.urukul0_ch1.set(100*MHz, amplitude=1.0)
        self.urukul0_ch2.set(100*MHz, amplitude=1.0)
        self.urukul0_ch3.set(100*MHz, amplitude=1.0)

    @kernel
    def force_update_ch0(self, freq_hz, amp, was_on):
        """Force update by toggling switch"""
        self.core.break_realtime()
        
        # Turn off
        self.urukul0_ch0.sw.off()
        delay(1*us)
        
        # Set new values
        self.urukul0_ch0.set(freq_hz, amplitude=amp)
        delay(1*us)
        
        # Turn back on if it was on
        if was_on:
            self.urukul0_ch0.sw.on()

    @kernel
    def force_update_ch1(self, freq_hz, amp, was_on):
        """Force update by toggling switch"""
        self.core.break_realtime()
        
        self.urukul0_ch1.sw.off()
        delay(1*us)
        self.urukul0_ch1.set(freq_hz, amplitude=amp)
        delay(1*us)
        if was_on:
            self.urukul0_ch1.sw.on()

    @kernel
    def force_update_ch2(self, freq_hz, amp, was_on):
        """Force update by toggling switch"""
        self.core.break_realtime()
        
        self.urukul0_ch2.sw.off()
        delay(1*us)
        self.urukul0_ch2.set(freq_hz, amplitude=amp)
        delay(1*us)
        if was_on:
            self.urukul0_ch2.sw.on()

    @kernel
    def force_update_ch3(self, freq_hz, amp, was_on):
        """Force update by toggling switch"""
        self.core.break_realtime()
        
        self.urukul0_ch3.sw.off()
        delay(1*us)
        self.urukul0_ch3.set(freq_hz, amplitude=amp)
        delay(1*us)
        if was_on:
            self.urukul0_ch3.sw.on()

    # Simple switch controls
    @kernel
    def switch_ch0_on(self):
        self.core.break_realtime()
        self.urukul0_ch0.sw.on()
        
    @kernel
    def switch_ch0_off(self):
        self.core.break_realtime()
        self.urukul0_ch0.sw.off()
        
    @kernel
    def switch_ch1_on(self):
        self.core.break_realtime()
        self.urukul0_ch1.sw.on()
        
    @kernel
    def switch_ch1_off(self):
        self.core.break_realtime()
        self.urukul0_ch1.sw.off()
        
    @kernel
    def switch_ch2_on(self):
        self.core.break_realtime()
        self.urukul0_ch2.sw.on()
        
    @kernel
    def switch_ch2_off(self):
        self.core.break_realtime()
        self.urukul0_ch2.sw.off()
        
    @kernel
    def switch_ch3_on(self):
        self.core.break_realtime()
        self.urukul0_ch3.sw.on()
        
    @kernel
    def switch_ch3_off(self):
        self.core.break_realtime()
        self.urukul0_ch3.sw.off()

    # RPC methods
    def set_frequency(self, channel, freq_mhz):
        """Set frequency - forces hardware update"""
        freq_hz = freq_mhz * 1e6
        self.current_freq[channel] = freq_hz
        
        # Force update with switch toggle
        if channel == 0:
            self.force_update_ch0(freq_hz, self.current_amp[0], self.current_sw[0])
        elif channel == 1:
            self.force_update_ch1(freq_hz, self.current_amp[1], self.current_sw[1])
        elif channel == 2:
            self.force_update_ch2(freq_hz, self.current_amp[2], self.current_sw[2])
        else:
            self.force_update_ch3(freq_hz, self.current_amp[3], self.current_sw[3])
            
        return f"CH{channel}={freq_mhz}MHz"

    def set_amplitude(self, channel, amplitude):
        """Set amplitude - forces hardware update"""
        self.current_amp[channel] = amplitude
        
        # Force update with switch toggle
        if channel == 0:
            self.force_update_ch0(self.current_freq[0], amplitude, self.current_sw[0])
        elif channel == 1:
            self.force_update_ch1(self.current_freq[1], amplitude, self.current_sw[1])
        elif channel == 2:
            self.force_update_ch2(self.current_freq[2], amplitude, self.current_sw[2])
        else:
            self.force_update_ch3(self.current_freq[3], amplitude, self.current_sw[3])
            
        return f"CH{channel}={int(amplitude*100)}%"

    def rf_on(self, channel):
        """Turn RF on - simple switch"""
        self.current_sw[channel] = True
        
        if channel == 0:
            self.switch_ch0_on()
        elif channel == 1:
            self.switch_ch1_on()
        elif channel == 2:
            self.switch_ch2_on()
        else:
            self.switch_ch3_on()
            
        return f"CH{channel} ON"

    def rf_off(self, channel):
        """Turn RF off - simple switch"""
        self.current_sw[channel] = False
        
        if channel == 0:
            self.switch_ch0_off()
        elif channel == 1:
            self.switch_ch1_off()
        elif channel == 2:
            self.switch_ch2_off()
        else:
            self.switch_ch3_off()
            
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
        print("Urukul Server - Force Update Version")
        print("=" * 50)
        print("Port: 3250")
        print("Forces hardware update by toggling switch")
        print("=" * 50)

        simple_server_loop({"urukul": self}, "0.0.0.0", 3250)
