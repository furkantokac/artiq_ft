from artiq.experiment import *

class FindUserLEDs(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        
        # Test different channel ranges for user LEDs
        # Usually user LEDs are at the end of the channel map
        self.test_channels = [48, 49, 50, 51, 52, 53, 54, 55]

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== FINDING USER LEDs ===")
        print("Testing channels one by one...")
        print("Watch your board - identify which LEDs blink!")
        delay(2000*ms)
        
        for channel in self.test_channels:
            print("Testing channel", channel)
            print("  Channel", channel, "ON for 2 seconds...")
            delay(2000*ms)  # 2 seconds ON
            print("  Channel", channel, "OFF for 1 second...")
            delay(1000*ms)  # 1 second OFF
            print("  Did you see LED", channel, "blink? Note it down!")
            delay(500*ms)
            
        print("=== TEST COMPLETED ===")
        print("Which channels made LEDs blink?")
        print("Update your device_db with correct channels!")
        delay(10*ms)
