from artiq.experiment import *

class FtArtiq(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        #self.urukul2_build()
        self.dash_build()

    @kernel
    def run(self):
        self.core.reset()
        #self.urukul1_run()
        self.dash_run()
        delay(1000*ms)

    ###
    ### Urukul1
    ###
    def urukul1_build(self):
        self.setattr_device("urukul0_ch1")

    @kernel
    def urukul1_run(self):
        self.urukul0_ch0.cpld.init() # initialises CPLD on channel 1
        self.urukul0_ch0.init() # initialises channel 1
        delay(10 * ms) # 10ms delay

        freq = 100 * MHz  # defines frequency variable
        amp = 1.0  # defines amplitude variable as an amplitude scale factor(0 to 1)
        attenuation = 1.0  # defines attenuation variable

        self.urukul0_ch0.set_att(attenuation)  # writes attenuation to urukul channel
        self.urukul0_ch0.sw.on()  # switches urukul channel on

        self.urukul0_ch0.set(freq, amplitude=amp)  # writes frequency and amplitude variables to urukul channel thus outputting function
        delay(20000 * ms)  # 2s delay
        self.urukul0_ch0.sw.off()  # switches urukul channel off

    ###
    ### Urukul2
    ###
    def urukul2_build(self):
        self.setattr_device("urukul0_cpld")
        # DDS kanallarını da ekleyin
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

    @kernel
    def urukul2_run(self):
        self.urukul0_cpld.init()
        delay(10*ms)

        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()

        self.urukul0_ch0.set(100 * MHz)
        self.urukul0_ch0.sw.on()

        delay(2000*ms)

    ###
    ### Protorev
    ###
    def protorev_build(self):
        self.setattr_device("spi_urukul0")

    @kernel
    def protorev_run(self):
        self.spi_urukul0.set_config_mu(
            SPI_CONFIG | spi2.SPI_END | spi2.SPI_INPUT,
            24, SPIT_CFG_RD, CS_CFG
        )

        cfg_reg = urukul_cfg(rf_sw=0, led=0, profile=7,
                             io_update=0, mask_nu=0, clk_sel=0,
                             sync_sel=0, rst=0, io_rst=0, clk_div=0)
        self.spi_urukul0.write(cfg_reg << 8)
        sta = self.spi_urukul0.read()

        proto_rev = urukul_sta_proto_rev(sta)
        print("Proto Rev:", proto_rev)

    ###
    ### Diag1
    ###
    def diag1_build(self):
        self.setattr_device("urukul0_cpld")

    @kernel
    def diag1_run(self):
        sta = self.urukul0_cpld.sta_read()
        print("CPLD STA =", sta)


    ###
    ### LED
    ###
    def leds_build(self):
        self.leds = []
        for i in range(8):
            self.setattr_device(f"led{i}")
            self.leds.append(self.get_device(f"led{i}"))

    @kernel
    def leds_run(self):
        self.leds_circle_loop()

    @kernel
    def leds_scan_loop(self):
        while 1:
            for i in range(8):
                self.leds[i].pulse(100*ms)

            for i in range(7, -1, -1):
                self.leds[i].pulse(100*ms)

            self.leds_toggle_n_times(3)

    @kernel
    def leds_circle_loop(self):
        while 1:
            for i in range(4):
                self.leds[i].pulse(100*ms)

            for i in range(7, 3, -1):
                self.leds[i].pulse(100 * ms)

    @kernel
    def leds_on(self):
        for i in range(8):
            self.leds[i].on()

    @kernel
    def leds_off(self):
        for i in range(8):
            self.leds[i].off()

    @kernel
    def leds_toggle_n_times(self, n=1):
        for _ in range(n):
            self.leds_on()
            delay(100*ms)
            self.leds_off()
            delay(100*ms)

    ###
    ### TTL Scan
    ###
    def ttl_scan_build(self):
        self.ttls = [self.get_device(f"ttl{i}") for i in range(24)]
        print(self.ttls)

    @kernel
    def ttl_scan_run(self):
        self.ttls[0].on()

    ###
    ### DASH
    ###
    def dash_build(self):
        """Basit dashboard kontrolleri"""
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

        # Basit kontroller
        self.setattr_argument("channel", NumberValue(0, min=0, max=3, step=1))
        self.setattr_argument("frequency_mhz", NumberValue(100.0, min=1.0, max=400.0))
        self.setattr_argument("power_percent", NumberValue(100.0, min=0.0, max=100.0))
        self.setattr_argument("rf_on", BooleanValue(True))

    @kernel
    def dash_run(self):
        """Hızlı çalıştırma"""
        # Kanal seç
        channels = [self.urukul0_ch0, self.urukul0_ch1,
                    self.urukul0_ch2, self.urukul0_ch3]
        ch = channels[int(self.channel)]

        # CPLD'yi kontrol et (init edilmemişse)
        try:
            ch.cpld.init(blind=True)
            ch.init()
        except:
            pass  # Zaten init edilmiş

        # Parametreleri ayarla
        amplitude = self.power_percent / 100.0
        ch.set(self.frequency_mhz * MHz, amplitude=amplitude)

        # RF kontrolü
        if self.rf_on:
            ch.sw.on()
        else:
            ch.sw.off()