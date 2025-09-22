from artiq.experiment import *


class AD9914Test(EnvExperiment):
    """AD9914 DDS RF test scripti - mevcut gateware için"""
    
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")  # Sadece led0 var
        
        # Mevcut çalışan AD9914 DDS devices
        self.setattr_device("ad9914dds0")  # DDS0 - RF çıkış
        self.setattr_device("ad9914dds1")  # DDS1 - RF çıkış

    @kernel
    def run(self):
        self.core.reset()
        print("AD9914 DDS RF Test başlıyor...")
        delay(10*ms)
        
        # LED'i aç - test başladığını göster
        self.led0.on()
        delay(500*ms)
        
        # DDS0 RF test
        print("DDS0: 100 MHz sinyal üretiliyor...")
        delay(1*ms)
        self.ad9914dds0.set(100*MHz)
        delay(3000*ms)  # 3 saniye bekle - spektrum analizerda görebilirsiniz
        
        # LED'i kısa süre söndür
        self.led0.off()
        delay(200*ms)
        self.led0.on()  # Tekrar aç
        
        # DDS1 RF test
        print("DDS1: 200 MHz sinyal üretiliyor...")
        delay(1*ms)
        self.ad9914dds1.set(200*MHz)
        delay(3000*ms)  # 3 saniye bekle
        
        # Frekansları değiştir
        print("DDS0: 150 MHz'e değiştiriliyor...")
        delay(1*ms)
        self.ad9914dds0.set(150*MHz)
        delay(1*ms)
        
        print("DDS1: 250 MHz'e değiştiriliyor...")
        delay(1*ms)
        self.ad9914dds1.set(250*MHz)
        delay(3000*ms)  # 3 saniye bekle
        
        # DDS'leri kapat (0 Hz'e set)
        print("DDS çıkışları kapatılıyor...")
        delay(1*ms)
        self.ad9914dds0.set(0*MHz)
        self.ad9914dds1.set(0*MHz)
        
        # LED'i söndür - test tamamlandı
        self.led0.off()
        delay(500*ms)
        
        print("AD9914 DDS RF Test tamamlandı!")
        print("Test edilen frekanslar:")
        print("- DDS0: 100MHz -> 150MHz")
        print("- DDS1: 200MHz -> 250MHz")
        delay(1*ms)
