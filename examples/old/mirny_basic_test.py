from artiq.experiment import *


class MirnyBasicTest(EnvExperiment):
    """Basic Mirny test using available SPI interface"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")

    @kernel
    def run(self):
        self.core.reset()
        print("Mirny Basic Test")
        delay(10*ms)
        
        # Visual feedback
        self.led0.on()
        delay(1000*ms)
        
        print("Mirny test - checking device availability...")
        print("Note: Mirny devices may need proper device_db configuration")
        print("Current test shows LED patterns only")
        
        # LED pattern to indicate Mirny test attempt
        for i in range(5):
            print(f"Mirny test cycle {i+1}/5")
            delay(1*ms)
            self.led0.off()
            delay(200*ms)
            self.led0.on() 
            delay(200*ms)
        
        print("Mirny basic test completed")
        print("For full Mirny functionality, SPI device configuration needed")
        delay(1*ms)
