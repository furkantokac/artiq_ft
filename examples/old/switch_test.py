from artiq.experiment import *


class SwitchTest(EnvExperiment):
    """Test the actual working switches instead of non-existent DDS"""
    
    def build(self):
        self.setattr_device("core")
        
        # Real working devices from gateware mapping
        self.setattr_device("led0")  # User LED 0
        self.setattr_device("led1")  # User LED 1

    @kernel
    def run(self):
        self.core.reset()
        print("=== SWITCH TEST (Real Hardware) ===")
        delay(10*ms)
        
        print("The gateware only has switches, no DDS channels!")
        print("Testing actual working devices...")
        delay(1*ms)
        
        # Test LED0 (User LED)
        print("Testing LED0 (User LED)...")
        for i in range(5):
            self.led0.on()
            delay(300*ms)
            self.led0.off()
            delay(300*ms)
        
        # Test LED1 (User LED)  
        print("Testing LED1 (User LED)...")
        for i in range(5):
            self.led1.on()
            delay(300*ms)
            self.led1.off()
            delay(300*ms)
        
        # Both LEDs on
        print("Both LEDs ON")
        self.led0.on()
        self.led1.on()
        delay(2000*ms)
        
        print("=== CONCLUSION ===")
        print("DDS devices are NOT implemented in this gateware!")
        print("Only switches and LEDs are available")
        print("For RF output, you need proper DDS gateware")
        
        delay(1*ms)
