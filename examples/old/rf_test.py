from artiq.experiment import *


class RFTest(EnvExperiment):
    """Urukul (AD9910 DDS) ve Mirny (ADF5356 PLL) RF test scripti"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        self.setattr_device("led1")
        
        # Urukul devices
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")  # RF0
        
        # Mirny devices  
        self.setattr_device("mirny0_cpld")
        self.setattr_device("mirny0_ch0")   # RF0

    @kernel
    def run(self):
        self.core.reset()
        print("RF Test başlıyor...")
        delay(10*ms)
        
        # LED'leri aç - test başladığını göster
        self.led0.on()
        self.led1.on()
        delay(500*ms)
        
        # Urukul initialization
        print("Urukul CPLD initialization...")
        delay(1*ms)
        self.urukul0_cpld.init()
        delay(100*ms)
        
        # Urukul RF0 (Channel 0) test
        print("Urukul RF0 DDS setup...")
        delay(1*ms)
        self.urukul0_ch0.init()
        delay(10*ms)
        
        # 100 MHz DDS frekansı set et
        print("Urukul RF0: 100 MHz sinyal üretiliyor...")
        delay(1*ms)
        self.urukul0_ch0.set(100*MHz)
        delay(1*ms)
        self.urukul0_ch0.sw.on()  # RF switch aç
        delay(2000*ms)  # 2 saniye bekle
        
        # LED0'ı söndür - Urukul test tamamlandı
        self.led0.off()
        delay(500*ms)
        
        # Mirny initialization  
        print("Mirny CPLD initialization...")
        delay(1*ms)
        self.mirny0_cpld.init()
        delay(100*ms)
        
        # Mirny RF0 (Channel 0) test
        print("Mirny RF0 PLL setup...")
        delay(1*ms)
        self.mirny0_ch0.init()
        delay(10*ms)
        
        # 1 GHz PLL frekansı set et
        print("Mirny RF0: 1 GHz frekans set ediliyor...")
        delay(1*ms)
        self.mirny0_ch0.set_frequency(1*GHz)
        delay(1*ms)
        self.mirny0_ch0.set_output_power(0)  # 0 dBm
        delay(2000*ms)  # 2 saniye bekle
        
        # LED1'i söndür - Mirny test tamamlandı
        self.led1.off()
        delay(500*ms)
        
        # RF'leri kapat
        print("RF çıkışları kapatılıyor...")
        delay(1*ms)
        self.urukul0_ch0.sw.off()
        self.mirny0_ch0.set_output_power(-20)  # Çıkışı azalt
        
        print("RF Test tamamlandı!")
        delay(1*ms)
