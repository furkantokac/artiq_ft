from artiq.experiment import *

class DirectTTLTest(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        
        # Test LEDs with known working channels from device_db
        self.setattr_device("led0")  # Channel 50
        self.setattr_device("led1")  # Channel 51  
        self.setattr_device("led2")  # Channel 52
        self.setattr_device("led3")  # Channel 53

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== DIRECT TTL LED TEST ===")
        print("Testing device_db LED definitions...")
        delay(1000*ms)
        
        print("Testing led0 (channel 50)...")
        for i in range(3):
            self.led0.on()
            delay(300*ms)
            self.led0.off()
            delay(300*ms)
        print("  led0 test completed")
        delay(1000*ms)
        
        print("Testing led1 (channel 51)...")
        for i in range(3):
            self.led1.on()
            delay(300*ms)
            self.led1.off()
            delay(300*ms)
        print("  led1 test completed")
        delay(1000*ms)
        
        print("Testing led2 (channel 52)...")
        for i in range(3):
            self.led2.on()
            delay(300*ms)
            self.led2.off()
            delay(300*ms)
        print("  led2 test completed")
        delay(1000*ms)
        
        print("Testing led3 (channel 53)...")
        for i in range(3):
            self.led3.on()
            delay(300*ms)
            self.led3.off()
            delay(300*ms)
        print("  led3 test completed")
        
        print("=== LED TEST COMPLETED ===")
        print("Which LEDs actually blinked on your board?")
        print("Tell me: led0, led1, led2, or led3?")
        delay(10*ms)
