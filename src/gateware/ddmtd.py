from migen import *
from migen.genlib.cdc import PulseSynchronizer, MultiReg
from misoc.interconnect.csr import *


class DDMTDSampler(Module):
    def __init__(self, cd_ref, main_clk_se):
        self.ref_beating = Signal()
        self.main_beating = Signal()

        # # #

        ref_beating_FF = Signal()
        main_beating_FF = Signal()
        self.specials += [
            # Two back to back FFs are used to prevent metastability
            Instance("FD", i_C=ClockSignal("helper"),
                     i_D=cd_ref.clk, o_Q=ref_beating_FF),
            Instance("FD", i_C=ClockSignal("helper"),
                     i_D=ref_beating_FF, o_Q=self.ref_beating),
            Instance("FD", i_C=ClockSignal("helper"),
                     i_D=main_clk_se, o_Q=main_beating_FF),
            Instance("FD", i_C=ClockSignal("helper"),
                     i_D=main_beating_FF, o_Q=self.main_beating)
        ]


class DDMTDDeglitcherFirstEdge(Module):
    def __init__(self, input_signal, blind_period=400):
        self.detect = Signal()
        rising = Signal()
        input_signal_r = Signal()

        # # #

        self.sync.helper += [
            input_signal_r.eq(input_signal),
            rising.eq(input_signal & ~input_signal_r)
        ]

        blind_counter = Signal(max=blind_period)
        self.sync.helper += [
            If(blind_counter != 0, blind_counter.eq(blind_counter - 1)),
            If(input_signal_r, blind_counter.eq(blind_period - 1)),
            self.detect.eq(rising & (blind_counter == 0))
        ]


class DDMTD(Module):
    def __init__(self, counter, input_signal):

        # in helper clock domain
        self.h_tag = Signal(len(counter))
        self.h_tag_update = Signal()

        # # #

        deglitcher = DDMTDDeglitcherFirstEdge(input_signal)
        self.submodules += deglitcher

        self.sync.helper += [
            self.h_tag_update.eq(0),
            If(deglitcher.detect,
                self.h_tag_update.eq(1),
                self.h_tag.eq(counter)
               )
        ]