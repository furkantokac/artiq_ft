from artiq.experiment import *


class DDSContinuousOutput(EnvExperiment):
    """AD9914 DDS continuous output for oscilloscope observation"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        
        # AD9914 DDS devices
        self.setattr_device("ad9914dds0")  # DDS0 - RF output
        self.setattr_device("ad9914dds1")  # DDS1 - RF output

    @kernel
    def run(self):
        self.core.reset()
        print("Setting up DDS for continuous output...")
        delay(10*ms)
        
        # Turn on LED to indicate test is running
        self.led0.on()
        delay(100*ms)
        
        # Set DDS0 to 100 MHz
        print("DDS0: Setting to 100 MHz - continuous output")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        delay(1*ms)
        
        # Set DDS1 to 200 MHz for comparison
        print("DDS1: Setting to 200 MHz - continuous output")
        delay(1*ms)
        self.ad9914dds1.set(200*MHz)
        delay(1*ms)
        
        print("DDS outputs are now active:")
        print("- DDS0 (RF0): 100 MHz continuous")
        print("- DDS1 (RF1): 200 MHz continuous")
        print("Connect oscilloscope to RF outputs to observe signals")
        print("LED0 is ON to indicate active output")
        print("Script completed - outputs remain active")
        delay(1*ms)
