from artiq.experiment import *


class DDSFullDiagnostic(EnvExperiment):
    """Complete DDS diagnostic to find why RF is not working"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Mirny RF2 control
        
        # Available DDS devices from working device_db
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel
    def run(self):
        self.core.reset()
        print("=== FULL DDS DIAGNOSTIC ===")
        delay(10*ms)
        
        # LED pattern - fast blink to show diagnostic start
        for i in range(10):
            self.led0.on()
            delay(50*ms)
            self.led0.off()
            delay(50*ms)
        
        print("Step 1: Testing DDS device responses...")
        delay(1*ms)
        
        # Test very low frequency first
        print("Setting DDS0 to 1 MHz...")
        delay(1*ms)
        self.ad9914dds0.set(1*MHz)
        delay(2000*ms)  # 2 seconds
        
        print("Setting DDS0 to 10 MHz...")
        delay(1*ms)
        self.ad9914dds0.set(10*MHz)
        delay(2000*ms)
        
        print("Setting DDS0 to 100 MHz...")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        delay(2000*ms)
        
        # Test DDS1
        print("Testing DDS1...")
        delay(1*ms)
        self.ad9914dds1.set(50*MHz)
        delay(2000*ms)
        
        # Test maximum frequency
        print("Testing high frequency: 500 MHz...")
        delay(1*ms)
        self.ad9914dds0.set(500*MHz)
        delay(2000*ms)
        
        # LED steady on to show diagnostic complete
        self.led0.on()
        
        print("=== DIAGNOSTIC COMPLETE ===")
        print("Frequencies tested:")
        print("- DDS0: 1MHz, 10MHz, 100MHz, 500MHz")
        print("- DDS1: 50MHz")
        print("")
        print("Check oscilloscope during each step")
        print("If still 50Hz only = Hardware/gateware issue")
        print("Expected: Clear frequency changes on RF outputs")
        
        delay(1*ms)
