from migen import *
from migen.genlib.cdc import MultiReg
from migen.genlib.resetsync import AsyncResetSynchronizer
from misoc.interconnect.csr import *


class ClockSwitchFSM(Module):
    def __init__(self):
        self.i_clk_sw = Signal()

        self.o_clk_sw = Signal()
        self.o_reset = Signal()

        ###

        i_switch = Signal()
        o_switch = Signal()
        reset = Signal()

        # at 125MHz bootstrap cd, will get around 0.5ms
        delay_counter = Signal(16, reset=0xFFFF)

        # register to prevent glitches
        self.sync.bootstrap += [
            self.o_clk_sw.eq(o_switch),
            self.o_reset.eq(reset),
        ]

        self.o_clk_sw.attr.add("no_retiming")
        self.o_reset.attr.add("no_retiming")
        self.i_clk_sw.attr.add("no_retiming")
        i_switch.attr.add("no_retiming")

        self.specials += MultiReg(self.i_clk_sw, i_switch, "bootstrap")

        fsm = ClockDomainsRenamer("bootstrap")(FSM(reset_state="START"))

        self.submodules += fsm

        fsm.act("START",
            If(i_switch & ~o_switch,
                NextState("RESET_START"))
        )
        
        fsm.act("RESET_START",
            reset.eq(1),
            If(delay_counter == 0,
                NextValue(delay_counter, 0xFFFF),
                NextState("CLOCK_SWITCH")
            ).Else(
                NextValue(delay_counter, delay_counter-1),
            )
        )

        fsm.act("CLOCK_SWITCH",
            reset.eq(1),
            NextValue(o_switch, 1),
            NextValue(delay_counter, delay_counter-1),
            If(delay_counter == 0,
                NextState("END"))
        )
        fsm.act("END",
            NextValue(o_switch, 1),
            reset.eq(0))


class SYSCRG(Module, AutoCSR):
    def __init__(self, platform, ps7, main_clk, clk_sw=None, freq=125e6):
        # assumes bootstrap clock is same freq as main and sys output
        self.clock_domains.cd_sys = ClockDomain()
        self.clock_domains.cd_sys4x = ClockDomain(reset_less=True)

        self.current_clock = CSRStatus()

        self.cd_sys.clk.attr.add("keep")

        bootstrap_clk = ClockSignal("bootstrap")

        period = 1e9/freq

        pll_locked = Signal()
        pll_sys = Signal()
        pll_sys4x = Signal()
        fb_clk = Signal()

        self.submodules.clk_sw_fsm = ClockSwitchFSM()

        if clk_sw is None:
            self.clock_switch = CSRStorage()
            self.comb += self.clk_sw_fsm.i_clk_sw.eq(self.clock_switch.storage)
        else:
            self.comb += self.clk_sw_fsm.i_clk_sw.eq(clk_sw)

        self.specials += [
            Instance("PLLE2_ADV",
                     p_STARTUP_WAIT="FALSE", o_LOCKED=pll_locked,
                     p_BANDWIDTH="HIGH",
                     p_REF_JITTER1=0.001,
                     p_CLKIN1_PERIOD=period, i_CLKIN1=main_clk,
                     p_CLKIN2_PERIOD=period, i_CLKIN2=bootstrap_clk,
                     i_CLKINSEL=self.clk_sw_fsm.o_clk_sw,

                     # VCO @ 1.5GHz when using 125MHz input
                     # 1.2GHz for 100MHz (zc706)
                     p_CLKFBOUT_MULT=12, p_DIVCLK_DIVIDE=1,
                     i_CLKFBIN=fb_clk,
                     i_RST=self.clk_sw_fsm.o_reset,

                     o_CLKFBOUT=fb_clk,

                     p_CLKOUT0_DIVIDE=3, p_CLKOUT0_PHASE=0.0,
                     o_CLKOUT0=pll_sys4x,

                     p_CLKOUT1_DIVIDE=12, p_CLKOUT1_PHASE=0.0,
                     o_CLKOUT1=pll_sys),
            Instance("BUFG", i_I=pll_sys, o_O=self.cd_sys.clk),
            Instance("BUFG", i_I=pll_sys4x, o_O=self.cd_sys4x.clk),

            AsyncResetSynchronizer(self.cd_sys, ~pll_locked),
        ]

        self.comb += self.current_clock.status.eq(self.clk_sw_fsm.o_clk_sw)
