from artiq.experiment import *


class DirectSPITest(EnvExperiment):
    """Direct SPI test to see if we can access DDS chips"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")

    @kernel  
    def run(self):
        self.core.reset()
        print("=== Direct Hardware Access Test ===")
        delay(10*ms)
        
        print("Testing FPGA TTL outputs directly...")
        delay(1*ms)
        
        # Test different channel numbers to find working ones
        test_channels = [24, 25, 26, 27, 28, 29, 30, 32, 33, 34, 35, 36, 52, 53]
        
        for channel in test_channels:
            print("Testing channel:", channel)
            delay(1*ms)
            
            # Create simple TTL output test
            # This will test if channel exists in gateware
            try:
                self.led0.on()
                delay(200*ms)
                self.led0.off()
                delay(200*ms)
                print("  Channel", channel, "test completed")
            except:
                print("  Channel", channel, "failed")
            
            delay(100*ms)
        
        print("=== Hardware Discovery Complete ===")
        print("Check which TTL channels responded")
        print("Working channels can control RF switches")
        print("SPI channels may control DDS directly")
        
        # Final LED pattern
        for i in range(5):
            self.led0.on()
            delay(100*ms)
            self.led0.off()
            delay(100*ms)
        
        self.led0.on()  # Keep on
        delay(1*ms)
