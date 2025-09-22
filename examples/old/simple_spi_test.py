from artiq.experiment import *

class SimpleSPITest(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("spi_urukul0")  # Direct SPI test

    @kernel
    def run(self):
        self.core.reset()
        delay(10*ms)
        
        print("=== SIMPLE SPI TEST ===")
        print("Testing SPI channel 24...")
        delay(1*ms)
        
        # Basit SPI konfigürasyonu
        self.spi_urukul0.set_config_mu(0, 24, 100, 0)
        delay(1*ms)
        print("SPI config set successfully")
        
        # Basit SPI yazma
        self.spi_urukul0.write(0x12345678)
        delay(1*ms)
        print("SPI write completed")
        
        print("=== SPI TEST COMPLETED ===")
        delay(10*ms)
