from artiq.experiment import *


class FinalDDSTest(EnvExperiment):
    """Final test using discovered working channels"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        self.setattr_device("urukul_dds")

    @kernel
    def run(self):
        self.core.reset()
        print("=== FINAL DDS TEST WITH WORKING CHANNELS ===")
        delay(10*ms)
        
        # LED to show test start
        self.led0.on()
        delay(500*ms)
        
        try:
            print("Testing AD9910 DDS on working channel 18...")
            delay(1*ms)
            
            # Try to set frequency
            self.urukul_dds.set(150*MHz)
            delay(1000*ms)
            
            print("DDS frequency set to 150 MHz!")
            print("Enabling RF switch...")
            delay(1*ms)
            
            self.urukul_dds.sw.on()
            delay(1000*ms)
            
            print("SUCCESS: DDS should be outputting 150 MHz!")
            print("Check RF output with oscilloscope NOW!")
            
            # LED fast blink for success
            for i in range(10):
                self.led0.off()
                delay(100*ms)
                self.led0.on()
                delay(100*ms)
            
        except Exception as e:
            print("DDS test failed - need proper CPLD configuration")
            
            # LED slow blink for failure
            for i in range(5):
                self.led0.off()
                delay(300*ms)
                self.led0.on()
                delay(300*ms)
        
        print("Test completed - LED indicates result")
        delay(1*ms)
