from artiq.experiment import *


class DDSDiagnostic(EnvExperiment):
    """Diagnostic script to understand DDS behavior"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # AD9914 DDS devices
        self.setattr_device("ad9914dds0")
        self.setattr_device("ad9914dds1")

    @kernel
    def run(self):
        self.core.reset()
        print("DDS Diagnostic Script")
        print("Checking AD9914 configuration...")
        delay(10*ms)
        
        # LED pattern for diagnostic start
        self.led0.on()
        delay(100*ms)
        self.led0.off()
        delay(100*ms)
        self.led0.on()
        delay(100*ms)
        
        print("AD9914 System Clock: 3 GHz (from device_db)")
        print("Bus Channel: 50")
        print("")
        
        # Test multiple frequencies to see the pattern
        frequencies = [50, 75, 100, 125, 150, 200]
        
        for freq in frequencies:
            print("Testing frequency:", freq, "MHz")
            delay(1*ms)
            
            # Set DDS0
            self.ad9914dds0.set(freq*MHz)
            delay(100*ms)
            
            # Brief LED blink for each frequency
            self.led0.off()
            delay(50*ms)
            self.led0.on()
            delay(50*ms)
            
            # Hold frequency for measurement
            delay(1500*ms)  # 1.5 seconds per frequency
        
        print("")
        print("Diagnostic complete. Final settings:")
        print("DDS0: 150 MHz")
        print("DDS1: 150 MHz")
        delay(1*ms)
        
        # Set final frequencies
        self.ad9914dds0.set(150*MHz)
        self.ad9914dds1.set(150*MHz)
        
        print("Check oscilloscope for actual output frequency")
        print("If frequency is wrong, there may be a PLL/clock issue")
        
        delay(1*ms)
