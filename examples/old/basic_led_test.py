from artiq.experiment import *

class BasicLEDTest(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Basic LED test

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== BASIC LED TEST ===")
        print("Testing LED control...")
        delay(1*ms)
        
        for i in range(5):
            print("LED ON", i+1)
            self.led0.on()
            delay(500*ms)
            
            print("LED OFF", i+1)
            self.led0.off()
            delay(500*ms)
        
        print("=== LED TEST COMPLETED ===")
        delay(10*ms)
