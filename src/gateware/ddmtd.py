from migen import *
from migen.genlib.cdc import PulseSynchronizer, MultiReg
from misoc.interconnect.csr import *


class DDMTDSampler(Module):
    def __init__(self, cd_ref, main_clk_se):
        self.ref_beating = Signal()
        self.main_beating = Signal()

        # # #

        ref_clk = Signal()
        self.specials +=[
            # ISERDESE2 can only be driven from fabric via IDELAYE2 (see UG471)
            Instance("IDELAYE2",
                    p_DELAY_SRC="DATAIN",
                    p_HIGH_PERFORMANCE_MODE="TRUE",
                    p_REFCLK_FREQUENCY=208.3,   # REFCLK frequency from IDELAYCTRL
                    p_IDELAY_VALUE=0,

                    i_DATAIN=cd_ref.clk,

                    o_DATAOUT=ref_clk
            ),
            Instance("ISERDESE2",
                    p_IOBDELAY="IFD",   # use DDLY as input
                    p_DATA_RATE="SDR",
                    p_DATA_WIDTH=2,     # min is 2
                    p_NUM_CE=1,

                    i_DDLY=ref_clk,
                    i_CE1=1,
                    i_CLK=ClockSignal("helper"),
                    i_CLKDIV=ClockSignal("helper"),

                    o_Q1=self.ref_beating
            ),
            Instance("ISERDESE2",
                    p_DATA_RATE="SDR",
                    p_DATA_WIDTH=2,     # min is 2
                    p_NUM_CE=1,

                    i_D=main_clk_se,
                    i_CE1=1,
                    i_CLK=ClockSignal("helper"),
                    i_CLKDIV=ClockSignal("helper"),

                    o_Q1=self.main_beating,
            ),
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