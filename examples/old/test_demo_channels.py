from artiq.experiment import *


class TestDemoChannels(EnvExperiment):
    """Test channels that would exist in demo.json gateware"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Working LED
        self.setattr_device("urukul_test")  # Channel 18 - Urukul start
        self.setattr_device("mirny_test")   # Channel 30 - Mirny start

    @kernel
    def run(self):
        self.core.reset()
        print("=== TESTING DEMO GATEWARE CHANNELS ===")
        delay(10*ms)
        
        print("Testing channels from demo.json build output:")
        print("- Urukul starts at channel 18 (0x000012)")
        print("- Mirny starts at channel 30 (0x00001e)")
        delay(1*ms)
        
        # Test working LED first
        print("Testing working LED (channel 35)...")
        for i in range(3):
            self.led0.on()
            delay(200*ms)
            self.led0.off()
            delay(200*ms)
        
        # Test Urukul channel
        print("Testing Urukul channel 18...")
        try:
            for i in range(3):
                self.urukul_test.on()
                delay(300*ms)
                self.urukul_test.off()
                delay(300*ms)
            print("  Urukul channel 18: SUCCESS")
        except:
            print("  Urukul channel 18: FAILED")
        
        # Test Mirny channel
        print("Testing Mirny channel 30...")
        try:
            for i in range(3):
                self.mirny_test.on()
                delay(300*ms)
                self.mirny_test.off()
                delay(300*ms)
            print("  Mirny channel 30: SUCCESS")
        except:
            print("  Mirny channel 30: FAILED")
        
        # Final indicator
        self.led0.on()
        
        print("=== CHANNEL TEST COMPLETE ===")
        print("If Urukul/Mirny channels work:")
        print("- Current gateware may have partial DDS support")
        print("- Flash may not be needed")
        print("If they fail:")
        print("- Need to flash demo.json gateware")
        
        delay(1*ms)
