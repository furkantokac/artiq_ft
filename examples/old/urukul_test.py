from artiq.experiment import *


class UrukulTest(EnvExperiment):
    """Sadece Urukul (AD9910 DDS) RF test scripti"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")
        self.setattr_device("led1")
        
        # Urukul devices
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")  # RF0

    @kernel
    def run(self):
        self.core.reset()
        print("Urukul RF Test başlıyor...")
        delay(10*ms)
        
        # LED'i aç - test başladığını göster
        self.led0.on()
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
        delay(3000*ms)  # 3 saniye bekle - spektrum analizerda görebilirsiniz
        
        # Frekansı değiştir
        print("Urukul RF0: 200 MHz'e değiştiriliyor...")
        delay(1*ms)
        self.urukul0_ch0.set(200*MHz)
        delay(3000*ms)  # 3 saniye bekle
        
        # RF'i kapat
        print("Urukul RF çıkışı kapatılıyor...")
        delay(1*ms)
        self.urukul0_ch0.sw.off()
        
        # LED'i söndür - test tamamlandı
        self.led0.off()
        delay(500*ms)
        
        print("Urukul RF Test tamamlandı!")
        delay(1*ms)
