from artiq.experiment import *


class Quick150MHzTest(EnvExperiment):
    """Quick 150MHz test with fixed device database"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        self.setattr_device("dds0")  # Working channel 18
        self.setattr_device("dds1")  # Working channel 30
        self.setattr_device("rf_switch0")
        self.setattr_device("rf_switch1")

    @kernel
    def run(self):
        self.core.reset()
        print("=== QUICK 150MHz TEST ===")
        delay(10*ms)
        
        # LED fast blink to show start
        for i in range(3):
            self.led0.on()
            delay(100*ms)
            self.led0.off()
            delay(100*ms)
        
        print("Setting DDS0 (channel 18) to 150 MHz...")
        delay(1*ms)
        self.dds0.set(150*MHz)
        delay(500*ms)
        
        print("Setting DDS1 (channel 30) to 200 MHz...")
        delay(1*ms)
        self.dds1.set(200*MHz)
        delay(500*ms)
        
        print("Enabling RF switches...")
        delay(1*ms)
        self.rf_switch0.on()  # Enable RF0
        self.rf_switch1.on()  # Enable RF1
        delay(500*ms)
        
        # LED solid on for success
        self.led0.on()
        
        print("=== QUICK TEST COMPLETE ===")
        print("DDS0: 150 MHz (channel 18)")
        print("DDS1: 200 MHz (channel 30)")
        print("RF switches: ENABLED")
        print("CHECK OSCILLOSCOPE NOW!")
        print("Expected: 150MHz and 200MHz signals")
        
        delay(1*ms)
