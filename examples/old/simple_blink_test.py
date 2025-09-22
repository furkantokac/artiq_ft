from artiq.experiment import *


class SimpleBlink(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")

    @kernel
    def run(self):
        self.core.reset()
        print("Starting LED blink test...")
        delay(1*ms)  # timing slack
        
        # 10 kez yanıp söndür
        for i in range(10):
            print("Blink", i+1, "/10")
            delay(1*ms)  # timing slack after print
            self.led0.on()
            delay(200*ms)
            self.led0.off()
            delay(200*ms)
        
        print("LED test completed!")
