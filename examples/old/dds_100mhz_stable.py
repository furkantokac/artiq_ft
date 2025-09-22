from artiq.experiment import *


class DDS100MHzStable(EnvExperiment):
    """Set both DDS outputs to 100 MHz and keep them stable"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # AD9914 DDS devices
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel
    def run(self):
        self.core.reset()
        print("Setting DDS outputs to 100 MHz...")
        delay(10*ms)
        
        # Turn on LED to indicate active output
        self.led0.on()
        delay(100*ms)
        
        # Set both DDS to 100 MHz
        print("DDS0: Setting to 100 MHz")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        delay(100*ms)
        
        print("DDS1: Setting to 100 MHz")
        delay(1*ms)
        self.ad9914dds1.set(100*MHz)
        delay(100*ms)
        
        print("Both DDS outputs now at 100 MHz")
        print("LED0 ON indicates active outputs")
        print("Measure differential signal:")
        print("- Probe tip to SMA center")
        print("- Probe GND to SMA shield")
        print("Expected: 100 MHz sine wave")
        
        delay(1*ms)
