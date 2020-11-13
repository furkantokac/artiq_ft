#!/usr/bin/env python

import argparse
from operator import itemgetter

from migen import *
from migen.build.generic_platform import *
from migen.genlib.resetsync import AsyncResetSynchronizer
from migen.genlib.cdc import MultiReg
from migen_axi.integration.soc_core import SoCCore
from migen_axi.platforms import redpitaya
from misoc.interconnect.csr import *
from misoc.integration import cpu_interface

from artiq.gateware import rtio
from artiq.gateware.rtio.phy import ttl_simple, ttl_serdes_7series, dds, spi2

import dma
import analyzer
import acpki


class RTIOCRG(Module, AutoCSR):
    def __init__(self, platform, rtio_internal_clk):
        self.clock_sel = CSRStorage()
        self.pll_reset = CSRStorage(reset=1)
        self.pll_locked = CSRStatus()
        self.clock_domains.cd_rtio = ClockDomain()
        self.clock_domains.cd_rtiox4 = ClockDomain(reset_less=True)

        rtio_external_clk = Signal()
        # user_sma_clock = platform.request("user_sma_clock")
        # platform.add_period_constraint(user_sma_clock.p, 8.0)
        # self.specials += Instance("IBUFDS",
        #                           i_I=user_sma_clock.p, i_IB=user_sma_clock.n,
        #                           o_O=rtio_external_clk)

        pll_locked = Signal()
        rtio_clk = Signal()
        rtiox4_clk = Signal()
        self.specials += [
            Instance("PLLE2_ADV",
                     p_STARTUP_WAIT="FALSE", o_LOCKED=pll_locked,

                     p_REF_JITTER1=0.01,
                     p_CLKIN1_PERIOD=8.0, p_CLKIN2_PERIOD=8.0,
                     i_CLKIN1=rtio_internal_clk, i_CLKIN2=rtio_external_clk,
                     # Warning: CLKINSEL=0 means CLKIN2 is selected
                     i_CLKINSEL=~self.clock_sel.storage,

                     # VCO @ 1GHz when using 125MHz input
                     p_CLKFBOUT_MULT=8, p_DIVCLK_DIVIDE=1,
                     i_CLKFBIN=self.cd_rtio.clk,
                     i_RST=self.pll_reset.storage,

                     o_CLKFBOUT=rtio_clk,

                     p_CLKOUT0_DIVIDE=2, p_CLKOUT0_PHASE=0.0,
                     o_CLKOUT0=rtiox4_clk),
            Instance("BUFG", i_I=rtio_clk, o_O=self.cd_rtio.clk),
            Instance("BUFG", i_I=rtiox4_clk, o_O=self.cd_rtiox4.clk),
            AsyncResetSynchronizer(self.cd_rtio, ~pll_locked),
            MultiReg(pll_locked, self.pll_locked.status)
        ]


class Redpitaya(SoCCore):
    def __init__(self, acpki=False):
        self.acpki = acpki
        self.rustc_cfg = dict()

        platform = redpitaya.Platform()
        platform.toolchain.bitstream_commands.extend([
            "set_property BITSTREAM.GENERAL.COMPRESS True [current_design]",
        ])
        ident = self.__class__.__name__
        if self.acpki:
            ident = "acpki_" + ident
        SoCCore.__init__(self, platform=platform, csr_data_width=32, ident=ident)

        platform.add_platform_command("create_clock -name clk_fpga_0 -period 8 [get_pins \"PS7/FCLKCLK[0]\"]")
        platform.add_platform_command("set_input_jitter clk_fpga_0 0.24")

        self.submodules.rtio_crg = RTIOCRG(self.platform, self.ps7.cd_sys.clk)
        self.csr_devices.append("rtio_crg")
        self.platform.add_period_constraint(self.rtio_crg.cd_rtio.clk, 8.)
        self.platform.add_false_path_constraints(
            self.ps7.cd_sys.clk,
            self.rtio_crg.cd_rtio.clk)

    def add_rtio(self, rtio_channels):
        self.submodules.rtio_tsc = rtio.TSC("async", glbl_fine_ts_width=3)
        self.submodules.rtio_core = rtio.Core(self.rtio_tsc, rtio_channels)
        self.csr_devices.append("rtio_core")

        if self.acpki:
            self.rustc_cfg["ki_impl"] = "acp"
            self.submodules.rtio = acpki.KernelInitiator(self.rtio_tsc,
                                                         bus=self.ps7.s_axi_acp,
                                                         user=self.ps7.s_axi_acp_user,
                                                         evento=self.ps7.event.o)
            self.csr_devices.append("rtio")
        else:
            self.rustc_cfg["ki_impl"] = "csr"
            self.submodules.rtio = rtio.KernelInitiator(self.rtio_tsc, now64=True)
            self.csr_devices.append("rtio")

        self.submodules.rtio_dma = dma.DMA(self.ps7.s_axi_hp0)
        self.csr_devices.append("rtio_dma")

        self.submodules.cri_con = rtio.CRIInterconnectShared(
            [self.rtio.cri, self.rtio_dma.cri],
            [self.rtio_core.cri])
        self.csr_devices.append("cri_con")

        self.submodules.rtio_moninj = rtio.MonInj(rtio_channels)
        self.csr_devices.append("rtio_moninj")

        self.submodules.rtio_analyzer = analyzer.Analyzer(self.rtio_tsc, self.rtio_core.cri,
                                                          self.ps7.s_axi_hp1)
        self.csr_devices.append("rtio_analyzer")


class Simple(Redpitaya):
    def __init__(self, **kwargs):
        Redpitaya.__init__(self, **kwargs)

        platform = self.platform

        rtio_channels = []
        for i in range(2):
            phy = ttl_simple.Output(platform.request("user_led", i))
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(phy))

        self.config["RTIO_LOG_CHANNEL"] = len(rtio_channels)
        rtio_channels.append(rtio.LogChannel())

        self.add_rtio(rtio_channels)


VARIANTS = {cls.__name__.lower(): cls for cls in [Simple]}


def write_csr_file(soc, filename):
    with open(filename, "w") as f:
        f.write(cpu_interface.get_csr_rust(
            soc.get_csr_regions(), soc.get_csr_groups(), soc.get_constants()))


def write_rustc_cfg_file(soc, filename):
    with open(filename, "w") as f:
        for k, v in sorted(soc.rustc_cfg.items(), key=itemgetter(0)):
            if v is None:
                f.write("{}\n".format(k))
            else:
                f.write("{}=\"{}\"\n".format(k, v))


def main():
    parser = argparse.ArgumentParser(
        description="ARTIQ port to the Redpitaya Zynq development kit")
    parser.add_argument("-r", default=None,
        help="build Rust interface into the specified file")
    parser.add_argument("-c", default=None,
        help="build Rust compiler configuration into the specified file")
    parser.add_argument("-g", default=None,
        help="build gateware into the specified directory")
    parser.add_argument("-V", "--variant", default="10",
                        help="variant: "
                             "[acpki_]simple "
                             "(default: %(default)s)")
    args = parser.parse_args()

    variant = args.variant.lower()
    acpki = variant.startswith("acpki_")
    if acpki:
        variant = variant[6:]
    soc = Simple(acpki=acpki)
    soc.finalize()

    if args.r is not None:
        write_csr_file(soc, args.r)
    if args.c is not None:
        write_rustc_cfg_file(soc, args.c)
    if args.g is not None:
        soc.build(build_dir=args.g)


if __name__ == "__main__":
    main()
