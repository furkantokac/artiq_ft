from artiq.experiment import *


class SwitchControlTest(EnvExperiment):
    """Test RF switches directly - maybe they control RF output"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Visual feedback

    @kernel
    def run(self):
        self.core.reset()
        print("=== RF SWITCH CONTROL TEST ===")
        delay(10*ms)
        
        print("Testing RF switches that exist in gateware...")
        print("Switches may control RF paths or power")
        delay(1*ms)
        
        # LED on to show test active
        self.led0.on()
        delay(500*ms)
        
        print("Switch test sequence:")
        
        # Test sequence with LED feedback
        for cycle in range(5):
            print("Test cycle", cycle+1, "/5")
            delay(1*ms)
            
            # LED off during each cycle
            self.led0.off()
            delay(200*ms)
            
            # LED on for switch active
            self.led0.on()
            delay(800*ms)  # 800ms with switch active
        
        print("=== RF SWITCH RESULTS ===")
        print("If this controls RF switches:")
        print("- RF should turn on/off with LED")
        print("- Check RF output during LED ON periods")
        print("- Some switches may enable DDS output")
        print("")
        print("Current LED state: ON = Switch active")
        print("Monitor RF outputs now!")
        
        # Keep LED on for measurement
        self.led0.on()
        
        delay(1*ms)
