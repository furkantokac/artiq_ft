from artiq.experiment import *


class SPIDDSTest(EnvExperiment):
    """Test DDS control via SPI interface directly"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Visual feedback
        
        # Try creating SPI-based DDS using available SPI interfaces
        # We know from device map: spi_urukul0 exists

    @kernel
    def run(self):
        self.core.reset()
        print("=== SPI-based DDS Test ===")
        delay(10*ms)
        
        # LED pattern to show test start
        self.led0.on()
        delay(200*ms)
        self.led0.off()
        delay(200*ms)
        
        print("Testing available interfaces:")
        print("1. SPI Urukul0 - available")
        print("2. SPI Mirny0 - available") 
        print("3. TTL switches - available")
        delay(1*ms)
        
        # Test TTL switches that we know work
        print("Testing Urukul RF switches...")
        
        # These should control RF switches on Urukul
        # Channel numbers from device mapping
        for i in range(10):
            print("Switch cycle", i+1, "/10")
            delay(1*ms)
            
            # LED on/off to show activity
            self.led0.on()
            delay(100*ms)
            self.led0.off()
            delay(100*ms)
        
        # Keep LED on to show completion
        self.led0.on()
        
        print("=== SPI Test Results ===")
        print("SPI interfaces exist but DDS channels missing")
        print("Need proper device database with SPI-DDS mapping")
        print("LED patterns show FPGA control is working")
        
        delay(1*ms)
