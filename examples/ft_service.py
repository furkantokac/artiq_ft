from artiq.experiment import *
from artiq.language.types import TBool, TFloat, TInt32
import threading
import time
from sipyco.pc_rpc import simple_server_loop
from artiq.sim.time import manager
import random

INDEX_IS_ON = 0
INDEX_FREQ = 1
INDEX_AMP = 2


class FastUrukulLoop(EnvExperiment):
    def build(self):
        ###
        ### Init class
        ###
        self.mirny_size = 4
        self.mirny_list = []

        self.urukul_size = 4
        self.urukul_devs = []
        self.urukul_freqs = []
        self.urukul_ampls = []
        self.urukul_sws = []

        ###
        ### Init Hardware
        ###
        self.setattr_device("core")

        self.setattr_device("urukul0_cpld")
        self.urukul_cp = self.get_device("urukul0_cpld")
        for i in range(self.urukul_size):
            self.setattr_device(f"urukul0_ch{i}")
            self.urukul_devs.append(self.get_device(f"urukul0_ch{i}"))
            self.urukul_freqs.append(100.0)
            self.urukul_ampls.append(1.0)
            self.setattr_device("ttl_urukul0_sw0")
            self.urukul_sws.append(self.get_device(f"ttl_urukul0_sw{i}"))

        self.setattr_device("mirny0_cpld")
        for i in range(self.mirny_size):
            self.setattr_device(f"mirny0_ch{i}")
            self.mirny_list.append(self.get_device(f"mirny0_ch{i}"))

        self.lock = threading.Lock()
        self.running = True

    @kernel
    def kernel_loop(self):
        self.core.reset()

        self.urukul_cp.init()
        for u in self.urukul_devs:
            u.init()
            u.set_att(10.0 * dB)
            u.sw.on()
            u.set(100.0 * MHz, amplitude=1.0)

        self.mirny0_cpld.init()

        i = 0
        while 1:
            freqs = self.get_urukul_freqs()
            amps = self.get_urukul_ampls()


            at_mu(now_mu() & ~7)
            for ch in range(4):
                u = self.urukul_devs[ch]
                ftw = u.frequency_to_ftw(freqs[ch] * MHz)
                asf = u.amplitude_to_asf(amps[ch])
                high = (asf << 16) | 0
                u.write64(0x0e + 7, high, ftw)
            self.urukul_cp.io_update.pulse_mu(32)


            self.ack_kernel_loop(now_mu())
            self.core.break_realtime()
            delay(5*ms)

            #self.urukul0_cpld.io_update.pulse(5 * ms)
            #self.core.wait_until_mu(now_mu())
            #self.core.reset()
            #for i in range(4):
            #    self.urukul_devs[i].set_mu(
            #        self.urukul_devs[i].frequency_to_ftw(freqs[i] * MHz),
            #        asf=self.urukul_devs[i].amplitude_to_asf(amps[i])
            #    )

    def get_urukul_freqs(self) -> TList(TFloat):
        with self.lock:
            return self.urukul_freqs

    def get_urukul_ampls(self) -> TList(TFloat):
        with self.lock:
            return self.urukul_ampls

    j = 0
    def ack_kernel_loop(self, time):
        self.j += 1
        if self.j >= 100:
            print("Kernel loop is OK.", time)
            self.j = 0

    def rpc_loop(self):
        print("=" * 50)
        print("Urukul, Mirny Server")
        print("=" * 50)
        print("Port: 3250")
        print("=" * 50)

        simple_server_loop({"urukul": self}, "0.0.0.0", 3250)
        """
        new_freq = 0.0
        while self.running:
            new_freq += 5.0
            if new_freq > 250.0: new_freq = 10.0
            with self.lock:
                self.target_freq_mhz = new_freq
            time.sleep(0.001)
        """

    def set_urukul_freq(self, channel, freq_mhz):
        with self.lock:
            self.urukul_freqs[channel] = freq_mhz

        return f"CH{channel}={freq_mhz}MHz"

    def set_urukul_ampl(self, channel, ampl):
        with self.lock:
            self.urukul_ampls[channel] = ampl

        return f"CH{channel}={ampl}%"

    def get_state(self, channel=None):
        if channel is not None:
            return {
                "freq_mhz": self.urukul_freqs[channel],
                "ampl": self.urukul_ampls[channel],
                "on": True
            }
        return {
            i: {
                "freq_mhz": self.urukul_freqs[i],
                "ampl": self.urukul_ampls[i],
                "on": True
            } for i in range(self.urukul_size)
        }

    def urukul_rf_on(self, channel):
        return True

    def urukul_rf_off(self, channel):
        return True

    def run(self):
        print("Starting kernel...")
        thread_kernel = threading.Thread(target=self.kernel_loop)
        thread_kernel.daemon = True
        thread_kernel.start()

        print("Starting RPC...")
        self.rpc_loop()


    @kernel
    def timing_test(self):
        print("Starting Urukul SW0 toggle test...")
        self.test_ttl.output()
        self.test_ttl.on()
        delay(1000*ms)

        print("\nTest1: starting 111...")
        t = now_mu()
        try:
            delay(1 * ns) # at_mu(t)
            self.test_ttl.on()
            delay(1 * ns) # at_mu(t)
            self.test_ttl.off()
            print("Test1: OK.")
        except:
            print("Test1: FAILED!")

        delay(10*ms)

        print("\nTest2: starting 222...")
        try:
            at_mu(t - 10000)
            self.test_ttl.on()
            at_mu(t - 10000)
            self.test_ttl.off()
            at_mu(t - 10000)
            print("Test2: FAILED!")
        except:
            print("Test2: OK.")

        print("\nTest3: starting 333...")
        try:
            for i in range(1000):
                #t = now_mu()

                #at_mu(t + 8)
                delay(1*ns)
                self.test_ttl.on()

                delay(1*ns)
                self.test_ttl.off()

                delay(255*us)
                #if i % 200 == 0:
                #    self.core.wait_until_mu(now_mu())
                #    t = now_mu()

            print("Test3: OK.")
        except RTIOUnderflow:
            print("Test3: FAILED! - underflow - 8-mu toggle error.")
        except RTIOOverflow:
            print("Test3: FAILED! - overflow - reduce loop size OR add wait_until_mu().")

        print("\n--- All tests are finished!")
        delay(5000*ms)