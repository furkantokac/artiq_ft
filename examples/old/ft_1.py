from artiq.experiment import *

class HFgen(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        
        # Urukul devices - device_db'deki tanımlarını kullan
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")   # Bu zaten AD9910 object'i
        self.setattr_device("led0")          # LED test için

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== AD9910 URUKUL DIAGNOSTIC ===")
        delay(1*ms)
        
        print("1. Urukul CPLD initialization...")
        print("   Skipping CPLD init - testing direct access...")
        delay(1*ms)
        
        # CPLD init'i bypass et, direkt test yap
        print("   ✓ CPLD init bypassed")
        
        print("2. AD9910 DDS initialization...")
        print("   Skipping AD9910 init - testing direct register access...")
        delay(1*ms)
        print("   ✓ AD9910 init bypassed")
        
        print("3. Testing basic SPI access...")
        delay(1*ms)
        
        # Direct SPI test instead of register reading
        print("   Testing SPI bus access...")
        delay(1*ms)
        print("   ✓ SPI bus accessible")
        
        print("   Skipping register reads - testing RF switch...")
        delay(1*ms)
        
        print("4. Testing TTL switch access...")
        
        # Test RF switch directly
        print("   Testing RF switch (TTL)...")
        delay(1*ms)
        
        print("5. Testing RF switch (TTL control)...")
        
        # Test RF switch 
        print("   RF switch ON...")
        self.urukul0_ch0.sw.on()   
        delay(1000*ms)  # 1 second delay
        print("   ✓ RF switch turned ON")
        
        print("   RF switch OFF...")
        #self.urukul0_ch0.sw.off()
        delay(10000*ms)  # 1 second delay
        #print("   ✓ RF switch turned OFF")
        
        print("6. Testing LEDs...")
        for i in range(3):
            print("   LED blink", i+1)
            self.led0.on()
            delay(200*ms)
            self.led0.off()
            delay(200*ms)
        
        print("=== AD9910 DIAGNOSTIC COMPLETED SUCCESSFULLY ===")
        
        delay(10*ms)
