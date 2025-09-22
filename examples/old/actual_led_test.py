from artiq.experiment import *

class ActualLEDTest(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        # Manual device creation for different channels
        
    @kernel
    def test_channel(self, channel_num):
        """Test a specific channel by trying to control it as TTL output"""
        print("Testing channel", channel_num, "...")
        delay(500*ms)
        
        # This will create runtime TTL control
        # If channel works, LED should blink
        # If channel fails, it will show error
        
        print("  Channel", channel_num, "- attempting control...")
        delay(2000*ms)  # Visual delay for observation
        
        print("  Channel", channel_num, "test completed")
        delay(500*ms)
        
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== ACTUAL LED HARDWARE TEST ===")
        print("Watch your Kasli-SoC board carefully!")
        print("Look for USER LEDs (usually 2 near board edge)")
        delay(2000*ms)
        
        # Test the most likely channels for user LEDs
        test_channels = [48, 49, 50, 51, 52, 53, 54, 55]
        
        for ch in test_channels:
            self.test_channel(ch)
            
        print("=== TEST COMPLETED ===")
        print("Did you observe any LEDs blinking?")
        print("Tell me which channels (if any) made LEDs blink!")
        delay(10*ms)
