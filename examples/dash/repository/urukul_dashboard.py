# repository/urukul_dashboard.py
from artiq.experiment import *
from artiq.language.units import MHz, dB, ms


class UrukulDashboard(EnvExperiment):
    """Urukul Dashboard Control"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

        # Dashboard GUI elemanları - TİP UYUMLU
        self.setattr_argument("action",
                              EnumerationValue(["Init", "Set", "On", "Off"],
                                               default="Set"),
                              "Control")

        self.setattr_argument("channel",
                              NumberValue(default=0, min=0, max=3, step=1, ndecimals=0),
                              "Channel")

        self.setattr_argument("frequency",
                              NumberValue(default=100.0, unit="MHz", min=1.0, max=400.0,
                                          step=0.1),
                              "Parameters")

        self.setattr_argument("amplitude",
                              NumberValue(default=1.0, min=0.0, max=1.0,
                                          step=0.01),
                              "Parameters")

        self.setattr_argument("attenuation",
                              NumberValue(default=10.0, unit="dB", min=0.0, max=31.5,
                                          step=0.5),
                              "Parameters")

    @kernel
    def run(self):
        self.core.reset()

        # Kanal seç
        ch_num = int(self.channel)  # Integer'a çevir
        channels = [self.urukul0_ch0, self.urukul0_ch1,
                    self.urukul0_ch2, self.urukul0_ch3]
        ch = channels[ch_num]

        # Action'a göre işlem yap
        if self.action == "Init":
            self.urukul0_cpld.init(blind=True)
            delay(10 * ms)
            for c in channels:
                c.init()
            delay(10 * ms)
            print("Urukul initialized")

        elif self.action == "Set":
            ch.set_att(self.attenuation)
            delay(1 * ms)
            ch.set(self.frequency * MHz, amplitude=self.amplitude)
            ch.sw.on()
            print("CH{ch_num}: {self.frequency}MHz @ {self.amplitude * 100}%")

        elif self.action == "On":
            ch.sw.on()
            print("CH{ch_num} ON")

        elif self.action == "Off":
            ch.sw.off()
            print("CH{ch_num} OFF")