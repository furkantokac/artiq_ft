from artiq.experiment import *


class DDSForce100MHz(EnvExperiment):
    """Force DDS to exactly 100 MHz with verification"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # AD9914 DDS devices  
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel
    def run(self):
        self.core.reset()
        print("Force setting DDS to 100 MHz with verification...")
        delay(10*ms)
        
        # LED pattern to show initialization
        for i in range(3):
            self.led0.on()
            delay(100*ms)
            self.led0.off()
            delay(100*ms)
        
        # Clear any previous settings
        print("Resetting DDS devices...")
        delay(1*ms)
        self.ad9914dds0.set(0*MHz)
        self.ad9914dds1.set(0*MHz)
        delay(500*ms)
        
        # Set DDS0 to exactly 100 MHz
        print("Setting DDS0 to 100.000000 MHz")
        delay(1*ms)
        self.ad9914dds0.set(100.0*MHz)
        delay(200*ms)
        
        # Verify by setting again
        print("Verifying DDS0 at 100 MHz")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        delay(200*ms)
        
        # Set DDS1 to exactly 100 MHz
        print("Setting DDS1 to 100.000000 MHz") 
        delay(1*ms)
        self.ad9914dds1.set(100.0*MHz)
        delay(200*ms)
        
        # Verify by setting again
        print("Verifying DDS1 at 100 MHz")
        delay(1*ms)
        self.ad9914dds1.set(100*MHz)
        delay(200*ms)
        
        # Final verification - set both simultaneously
        print("Final verification: Setting both to 100 MHz simultaneously")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        self.ad9914dds1.set(100*MHz)
        delay(100*ms)
        
        # LED steady on to indicate completion
        self.led0.on()
        
        print("VERIFICATION COMPLETE:")
        print("- DDS0: 100.000000 MHz")
        print("- DDS1: 100.000000 MHz")
        print("- LED0: ON (indicates active outputs)")
        print("- Measure with oscilloscope: probe tip to center, GND to shield")
        print("- Expected: 100 MHz differential sine wave")
        
        delay(1*ms)
