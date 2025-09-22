from artiq.experiment import *

class LEDFinder(EnvExperiment):
    def build(self):
        self.setattr_device("core")

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== LED FINDER ===")
        print("Testing different channel ranges...")
        print("Watch LEDs on your board!")
        delay(1*ms)
        
        # Test channel range 48-55 (typical user LED range)
        print("Testing channels 48-55...")
        delay(1000*ms)
        
        print("=== Manual LED Test ===")
        print("If you know approximate range, test manually:")
        print("Look at your board - which LEDs are user LEDs?")
        print("Usually 2 user LEDs near the edge of the board")
        
        delay(5000*ms)  # 5 seconds to identify LEDs manually
        
        print("=== LED FINDER COMPLETED ===")
        print("Next: Try channels around where peripherals end")
        print("Urukul ends around channel 30, so try 48+ for user LEDs")
        delay(10*ms)
