from artiq.experiment import *


class RealDDSTest(EnvExperiment):
    """Test real AD9910 DDS with minimal configuration"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # Real hardware devices
        self.setattr_device("urukul_cpld")
        self.setattr_device("dds0")

    @kernel
    def run(self):
        self.core.reset()
        print("=== REAL DDS TEST - AD9910 via SPI ===")
        delay(10*ms)
        
        # LED fast blink to show real test starting
        for i in range(3):
            self.led0.on()
            delay(100*ms)
            self.led0.off()
            delay(100*ms)
        
        try:
            print("Step 1: Initialize Urukul CPLD...")
            delay(1*ms)
            self.urukul_cpld.init()
            delay(500*ms)
            print("  CPLD init: SUCCESS")
            
            print("Step 2: Initialize AD9910 DDS...")
            delay(1*ms)
            self.dds0.init()
            delay(500*ms)
            print("  DDS init: SUCCESS")
            
            print("Step 3: Set frequency to 150 MHz...")
            delay(1*ms)
            self.dds0.set(150*MHz)
            delay(200*ms)
            print("  Frequency set: SUCCESS")
            
            print("Step 4: Enable RF switch...")
            delay(1*ms)
            self.dds0.sw.on()
            delay(200*ms)
            print("  RF switch: ON")
            
            # LED solid on for success
            self.led0.on()
            
            print("=== DDS TEST SUCCESS ===")
            print("AD9910 should now output 150 MHz!")
            print("Check RF output with oscilloscope")
            print("Expected: 150 MHz differential signal")
            
        except Exception as e:
            print("DDS Test failed at some step")
            print("Check gateware DDS support")
            
            # LED fast blink for error
            for i in range(10):
                self.led0.on()
                delay(50*ms)
                self.led0.off()
                delay(50*ms)
        
        delay(1*ms)
