from artiq.experiment import *

class MinimalRFOutput(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("led0")  # Channel 50 - çalışıyor
        self.setattr_device("led1")  # Channel 51 - çalışıyor

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== MINIMAL RF OUTPUT TEST ===")
        
        # LED indicators - starting (both LEDs on)
        self.led0.on()  # Indicates script running
        self.led1.on()  # Will indicate RF status
        delay(100*ms)
        
        print("1. Initializing CPLD (minimal)...")
        # Manuel CPLD register setup - minimal
        self.urukul0_cpld.cfg_write(0x000001)  # Minimal config
        delay(1*ms)
        print("   ✓ CPLD minimal setup done")
        
        print("2. Manual AD9910 setup...")
        # Manuel AD9910 register yazma - RF output için minimum
        
        # CFR1: DDS enable, basic setup
        self.urukul0_cpld.cfg_write(0x000001)  # Minimal config
        delay(1*ms)
        
        # Frequency: 100 MHz (FTW calculation)
        # FTW = freq * 2^32 / sysclk
        # sysclk yaklaşık 1 GHz için, 100 MHz = ~429496730
        ftw_100mhz = 429496730  # 100 MHz için approximate FTW
        self.urukul0_ch0.write32(0x07, ftw_100mhz)  # FTW register
        delay(1*ms)
        
        # Amplitude: Maximum
        self.urukul0_ch0.write32(0x09, 0x3FFF0000)  # ASF register - max amplitude
        delay(1*ms)
        
        # Phase: 0
        self.urukul0_ch0.write16(0x08, 0x0000)  # POW register
        delay(1*ms)
        
        print("3. Enabling RF output...")
        
        # RF switch ON
        self.urukul0_ch0.sw.on()
        delay(1*ms)
        
        # IO Update to apply settings
        self.urukul0_cpld.io_update.pulse(1*us)
        delay(1*ms)
        
        print("   ✓ RF output should be active at 100 MHz")
        print("   Check RF0 SMA connector with oscilloscope!")
        print("   LED0 = Script running, LED1 = RF status")
        
        # LED1 indicates RF is on, LED0 blinks for heartbeat
        self.led1.off()  # LED1 will pulse with RF status
        for i in range(10):
            # LED0 heartbeat (script running)
            self.led0.off()
            delay(100*ms)
            self.led0.on() 
            delay(100*ms)
            
            # LED1 RF status indicator
            self.led1.on()
            delay(100*ms)
            self.led1.off()
            delay(100*ms)
            
        print("4. Testing frequency changes...")
        
        # Test different frequencies
        print("   Setting 50 MHz...")
        self.urukul0_ch0.write32(0x07, 214748365)  # 50 MHz FTW
        self.urukul0_cpld.io_update.pulse(1*us)
        delay(2000*ms)
        
        print("   Setting 150 MHz...")
        self.urukul0_ch0.write32(0x07, 644245095)  # 150 MHz FTW
        self.urukul0_cpld.io_update.pulse(1*us)
        delay(2000*ms)
        
        print("   Setting 200 MHz...")
        self.urukul0_ch0.write32(0x07, 858993459)  # 200 MHz FTW
        self.urukul0_cpld.io_update.pulse(1*us)
        delay(2000*ms)
            
        print("5. RF output test completed")
        
        # Turn off RF
        self.urukul0_ch0.sw.off()
        self.led0.off()
        self.led1.off()
        
        print("=== RF TEST COMPLETED ===")
        print("Did you see RF signal on oscilloscope at RF0 connector?")
        print("Expected: 100 MHz, then 50, 150, 200 MHz")
        delay(10*ms)
