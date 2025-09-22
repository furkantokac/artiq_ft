from artiq.experiment import *


class Simple150MHzTest(EnvExperiment):
    """Simple test using existing working device database but targeting 150 MHz"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Use existing working LED
        
        # Use existing working AD9914 devices from original device_db
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel
    def run(self):
        self.core.reset()
        print("Simple 150 MHz Test with existing device configuration")
        delay(10*ms)
        
        # LED on to show test start
        self.led0.on()
        delay(500*ms)
        
        # Clear any previous settings
        print("Clearing previous DDS settings...")
        delay(1*ms)
        self.ad9914dds0.set(0*MHz)
        self.ad9914dds1.set(0*MHz)
        delay(1000*ms)
        
        # Set 150 MHz on both channels
        print("Setting DDS0 to 150 MHz...")
        delay(1*ms)
        self.ad9914dds0.set(150*MHz)
        delay(500*ms)
        
        print("Setting DDS1 to 150 MHz...")
        delay(1*ms)
        self.ad9914dds1.set(150*MHz)
        delay(500*ms)
        
        # LED blink pattern to confirm completion
        for i in range(3):
            self.led0.off()
            delay(200*ms)
            self.led0.on()
            delay(200*ms)
        
        print("Test complete:")
        print("- DDS0: 150 MHz")
        print("- DDS1: 150 MHz")
        print("- LED indicates active (may be Mirny RF2)")
        print("Measure RF outputs with oscilloscope")
        
        delay(1*ms)
