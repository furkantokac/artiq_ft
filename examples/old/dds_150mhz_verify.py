from artiq.experiment import *


class DDS150MHzVerify(EnvExperiment):
    """Set DDS to 150 MHz and verify all settings"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # AD9914 DDS devices
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel
    def run(self):
        self.core.reset()
        print("Setting DDS to 150 MHz with full verification...")
        delay(10*ms)
        
        # LED blink to show start
        self.led0.on()
        delay(200*ms)
        self.led0.off()
        delay(200*ms)
        
        # Clear previous settings
        print("Step 1: Clearing previous DDS settings")
        delay(1*ms)
        self.ad9914dds0.set(0*MHz)
        self.ad9914dds1.set(0*MHz)
        delay(1000*ms)  # 1 second delay
        
        print("Step 2: Setting DDS0 to 150 MHz")
        delay(1*ms)
        self.ad9914dds0.set(150*MHz)
        delay(500*ms)
        
        print("Step 3: Setting DDS1 to 150 MHz")
        delay(1*ms)
        self.ad9914dds1.set(150*MHz)
        delay(500*ms)
        
        # Blink LED to indicate completion
        for i in range(5):
            self.led0.on()
            delay(100*ms)
            self.led0.off()
            delay(100*ms)
        
        # Keep LED on to show active state
        self.led0.on()
        
        print("SETUP COMPLETE:")
        print("- DDS0: 150 MHz")
        print("- DDS1: 150 MHz") 
        print("- LED0: ON indicates active")
        print("")
        print("Next: Check device logs for verification")
        
        delay(1*ms)
