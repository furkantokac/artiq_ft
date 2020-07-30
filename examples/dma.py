from artiq.experiment import *

class DMAPulses(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("core_dma")
        self.setattr_device("led0")

    @kernel
    def record(self):
        with self.core_dma.record("pulse"):
            delay(200*ms)
            # all RTIO operations now go to the "pulse"
            # DMA buffer, instead of being executed immediately.
            self.led0.pulse(500*ms)


    @kernel
    def run(self):
        self.core.reset()
        self.record()
        # prefetch the address of the DMA buffer
        # for faster playback trigger
        pulse_handle = self.core_dma.get_handle("pulse")
        self.core.break_realtime()
        self.core_dma.playback_handle(pulse_handle)
