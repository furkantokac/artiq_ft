# repository/quick_presets.py
from artiq.experiment import *
from artiq.language.units import MHz, ms


class QuickPreset100MHz(EnvExperiment):
    """CH0 100MHz"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_ch0")

    @kernel
    def run(self):
        self.core.reset()
        self.urukul0_ch0.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch0.sw.on()


class QuickPreset200MHz(EnvExperiment):
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
    """All Channels Off"""

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