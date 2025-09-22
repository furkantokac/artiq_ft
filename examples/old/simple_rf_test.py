from artiq.experiment import *


class SimpleRFTest(EnvExperiment):
    """Simple test for both Urukul and Mirny - sweep frequencies"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # Available AD9914 DDS devices
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel  
    def run(self):
        self.core.reset()
        print("Simple RF Test - Frequency Sweep")
        delay(10*ms)
        
        # LED blink pattern to show test is running
        self.led0.on()
        delay(500*ms)
        
        # Test DDS0 with frequency sweep
        print("DDS0 frequency sweep starting...")
        for freq_mhz in [50, 100, 150, 200, 250]:
            print("DDS0:", freq_mhz, "MHz")
            delay(1*ms)
            self.ad9914dds0.set(freq_mhz*MHz)
            delay(2000*ms)  # 2 seconds per frequency
            
            # LED blink to show frequency change
            self.led0.off()
            delay(100*ms)
            self.led0.on()
            delay(100*ms)
        
        print("DDS0 sweep completed")
        delay(500*ms)
        
        # Test DDS1 with different frequencies
        print("DDS1 frequency sweep starting...")
        for freq_mhz in [75, 125, 175, 225, 275]:
            print("DDS1:", freq_mhz, "MHz")
            delay(1*ms)
            self.ad9914dds1.set(freq_mhz*MHz)
            delay(2000*ms)  # 2 seconds per frequency
            
            # LED blink pattern
            self.led0.off()
            delay(50*ms)
            self.led0.on()
            delay(50*ms)
            self.led0.off()
            delay(50*ms)
            self.led0.on()
            delay(50*ms)
        
        print("DDS1 sweep completed")
        
        # Final stable outputs for measurement
        print("Setting final test frequencies:")
        print("DDS0: 100 MHz (stable)")
        print("DDS1: 200 MHz (stable)")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        self.ad9914dds1.set(200*MHz)
        
        print("Test completed - outputs stable for measurement")
        print("Connect oscilloscope/spectrum analyzer to observe signals")
        delay(1*ms)
