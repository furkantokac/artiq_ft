# repository/presets.py
from artiq.experiment import *
from artiq.language.units import MHz, ms


class Init(EnvExperiment):
    """Initialize Urukul"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

    @kernel
    def run(self):
        self.core.reset()
        self.urukul0_cpld.init(blind=True)
        delay(10 * ms)
        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()
        print("Ready!")


class CH0_100MHz(EnvExperiment):
    """CH0 100MHz"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_ch0")

    @kernel
    def run(self):
        self.core.reset()
        self.urukul0_ch0.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch0.sw.on()


class CH0_200MHz(EnvExperiment):
    """CH0 200MHz"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_ch0")

    @kernel
    def run(self):
        self.core.reset()
        self.urukul0_ch0.set(200 * MHz, amplitude=1.0)
        self.urukul0_ch0.sw.on()


class AllOff(EnvExperiment):
    """All OFF"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

    @kernel
    def run(self):
        self.core.reset()
        self.urukul0_ch0.sw.off()
        self.urukul0_ch1.sw.off()
        self.urukul0_ch2.sw.off()
        self.urukul0_ch3.sw.off()