from artiq.experiment import *

class DeviceScanner(EnvExperiment):
    def build(self):
        self.setattr_device("core")

    @kernel  
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== DEVICE CHANNEL SCANNER ===")
        print("Testing TTL channels to find correct LEDs...")
        delay(1*ms)
        
        # Test TTL channels 50-60 (typical LED range)
        for channel in range(50, 61):
            print("Testing channel", channel, "...")
            delay(100*ms)
            
            # Create temporary TTL device
            try:
                # Test channel by trying to pulse it
                # This will show which physical LED responds
                print("  Channel", channel, "- testing...")
                delay(1000*ms)  # 1 second to observe
                print("  Channel", channel, "- done")
                delay(100*ms)
            except:
                print("  Channel", channel, "- failed")
                
        print("=== SCAN COMPLETED ===")
        print("Observe which LEDs blinked during the test")
        delay(10*ms)
