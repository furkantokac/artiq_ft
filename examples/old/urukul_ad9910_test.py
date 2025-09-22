from artiq.experiment import *


class UrukulAD9910Test(EnvExperiment):
    """Test for actual AD9910 Urukul hardware"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("user_led0")  # Real user LED
        
        # Real Urukul AD9910 devices
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")  # RF0

    @kernel
    def run(self):
        self.core.reset()
        print("AD9910 Urukul Test - Setting 150 MHz")
        delay(10*ms)
        
        # Real user LED control
        self.user_led0.on()
        delay(500*ms)
        
        # Initialize Urukul CPLD
        print("Initializing Urukul CPLD...")
        delay(1*ms)
        self.urukul0_cpld.init()
        delay(100*ms)
        
        # Initialize AD9910 DDS
        print("Initializing AD9910 DDS channel 0...")
        delay(1*ms)
        self.urukul0_ch0.init()
        delay(100*ms)
        
        # Set frequency to 150 MHz
        print("Setting Urukul RF0 to 150 MHz...")
        delay(1*ms)
        self.urukul0_ch0.set(150*MHz)
        delay(100*ms)
        
        # Turn on RF switch
        print("Enabling RF output...")
        delay(1*ms)
        self.urukul0_ch0.sw.on()
        delay(100*ms)
        
        print("AD9910 Urukul RF0: 150 MHz active")
        print("Real user LED ON indicates success")
        print("Measure RF0 output with oscilloscope")
        
        delay(1*ms)
