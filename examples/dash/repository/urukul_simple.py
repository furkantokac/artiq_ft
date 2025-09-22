# repository/urukul_simple.py
from artiq.experiment import *
from artiq.language.units import MHz, ms


class UrukulSimple(EnvExperiment):
    """Urukul Simple Control"""

    def build(self):
        self.setattr_device("core")
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

        # Basit string input kullan
        self.setattr_argument("command",
                              StringValue("set 0 100 1.0"),
                              "Command (action ch freq amp)")

    @kernel
    def run(self):
        self.core.reset()
        self.core.break_realtime()

        # Command'i parse et (host'ta)
        parts = self.command.split()
        if len(parts) < 1:
            return

        action = parts[0]

        if action == "init":
            self.init_all()
        elif action == "set" and len(parts) >= 4:
            ch = int(parts[1])
            freq = float(parts[2])
            amp = float(parts[3])
            self.set_channel(ch, freq, amp)
        elif action == "off" and len(parts) >= 2:
            ch = int(parts[1])
            self.off_channel(ch)

    @kernel
    def init_all(self):
        self.urukul0_cpld.init(blind=True)
        delay(10 * ms)
        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()

    @kernel
    def set_channel(self, ch, freq_mhz, amp):
        channels = [self.urukul0_ch0, self.urukul0_ch1,
                    self.urukul0_ch2, self.urukul0_ch3]
        if 0 <= ch < 4:
            channels[ch].set(freq_mhz * MHz, amplitude=amp)
            channels[ch].sw.on()

    @kernel
    def off_channel(self, ch):
        channels = [self.urukul0_ch0, self.urukul0_ch1,
                    self.urukul0_ch2, self.urukul0_ch3]
        if 0 <= ch < 4:
            channels[ch].sw.off()