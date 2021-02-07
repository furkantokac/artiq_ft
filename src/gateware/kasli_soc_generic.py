#!/usr/bin/env python

import argparse
from operator import itemgetter

from migen import *
from migen.build.generic_platform import *
from migen.genlib.resetsync import AsyncResetSynchronizer
from migen.genlib.cdc import MultiReg
from migen_axi.integration.soc_core import SoCCore
from migen_axi.platforms import kasli_soc
from misoc.interconnect.csr import *
from misoc.integration import cpu_interface

from artiq.coredevice import jsondesc
from artiq.gateware import rtio, eem_7series

import dma
import analyzer
import acpki


class RTIOCRG(Module, AutoCSR):
    def __init__(self, platform):
        self.pll_reset = CSRStorage(reset=1)
        self.pll_locked = CSRStatus()
        self.clock_domains.cd_rtio = ClockDomain()
        self.clock_domains.cd_rtiox4 = ClockDomain(reset_less=True)

        clk_synth = platform.request("cdr_clk_clean_fabric")
        clk_synth_se = Signal()
        platform.add_period_constraint(clk_synth.p, 8.0)
        self.specials += [
            Instance("IBUFGDS",
                p_DIFF_TERM="TRUE", p_IBUF_LOW_PWR="FALSE",
                i_I=clk_synth.p, i_IB=clk_synth.n, o_O=clk_synth_se),
        ]

        pll_locked = Signal()
        rtio_clk = Signal()
        rtiox4_clk = Signal()
        fb_clk = Signal()
        self.specials += [
            Instance("PLLE2_ADV",
                     p_STARTUP_WAIT="FALSE", o_LOCKED=pll_locked,
                     p_BANDWIDTH="HIGH",
                     p_REF_JITTER1=0.001,
                     p_CLKIN1_PERIOD=8.0, p_CLKIN2_PERIOD=8.0,
                     i_CLKIN2=clk_synth_se,
                     # Warning: CLKINSEL=0 means CLKIN2 is selected
                     i_CLKINSEL=0,

                     # VCO @ 1.5GHz when using 125MHz input
                     p_CLKFBOUT_MULT=12, p_DIVCLK_DIVIDE=1,
                     i_CLKFBIN=fb_clk,
                     i_RST=self.pll_reset.storage,

                     o_CLKFBOUT=fb_clk,

                     p_CLKOUT0_DIVIDE=3, p_CLKOUT0_PHASE=0.0,
                     o_CLKOUT0=rtiox4_clk,

                     p_CLKOUT1_DIVIDE=12, p_CLKOUT1_PHASE=0.0,
                     o_CLKOUT1=rtio_clk),
            Instance("BUFG", i_I=rtio_clk, o_O=self.cd_rtio.clk),
            Instance("BUFG", i_I=rtiox4_clk, o_O=self.cd_rtiox4.clk),

            AsyncResetSynchronizer(self.cd_rtio, ~pll_locked),
            MultiReg(pll_locked, self.pll_locked.status)
        ]


class GenericStandalone(SoCCore):
    def __init__(self, description, acpki=False):
        self.acpki = acpki
        self.rustc_cfg = dict()

        platform = kasli_soc.Platform()
        platform.toolchain.bitstream_commands.extend([
            "set_property BITSTREAM.GENERAL.COMPRESS True [current_design]",
        ])
        ident = self.__class__.__name__
        if self.acpki:
            ident = "acpki_" + ident
        SoCCore.__init__(self, platform=platform, csr_data_width=32, ident=ident)

        platform.add_platform_command("create_clock -name clk_fpga_0 -period 8 [get_pins \"PS7/FCLKCLK[0]\"]")
        platform.add_platform_command("set_input_jitter clk_fpga_0 0.24")

        self.crg = self.ps7 # HACK for eem_7series to find the clock
        self.submodules.rtio_crg = RTIOCRG(self.platform)
        self.csr_devices.append("rtio_crg")
        self.platform.add_period_constraint(self.rtio_crg.cd_rtio.clk, 8.)
        self.platform.add_false_path_constraints(
            self.ps7.cd_sys.clk,
            self.rtio_crg.cd_rtio.clk)

        self.rtio_channels = []
        has_grabber = any(peripheral["type"] == "grabber" for peripheral in description["peripherals"])
        if has_grabber:
            self.grabber_csr_group = []
        eem_7series.add_peripherals(self, description["peripherals"])

        self.submodules.rtio_tsc = rtio.TSC("async", glbl_fine_ts_width=3)
        self.submodules.rtio_core = rtio.Core(self.rtio_tsc, self.rtio_channels)
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

        self.submodules.rtio_moninj = rtio.MonInj(self.rtio_channels)
        self.csr_devices.append("rtio_moninj")

        self.submodules.rtio_analyzer = analyzer.Analyzer(self.rtio_tsc, self.rtio_core.cri,
                                                          self.ps7.s_axi_hp1)
        self.csr_devices.append("rtio_analyzer")

        if has_grabber:
            self.config["HAS_GRABBER"] = None
            self.add_csr_group("grabber", self.grabber_csr_group)
            for grabber in self.grabber_csr_group:
                self.platform.add_false_path_constraints(
                    self.rtio_crg.cd_rtio.clk, getattr(self, grabber).deserializer.cd_cl.clk)


class GenericMaster(SoCCore):
    def __init__(self, description, **kwargs):
        raise NotImplementedError


class GenericSatellite(SoCCore):
    def __init__(self, description, **kwargs):
        raise NotImplementedError


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
        description="ARTIQ device binary builder for generic Kasli-SoC systems")
    parser.add_argument("-r", default=None,
        help="build Rust interface into the specified file")
    parser.add_argument("-c", default=None,
        help="build Rust compiler configuration into the specified file")
    parser.add_argument("-g", default=None,
        help="build gateware into the specified directory")
    parser.add_argument("--acpki", default=False, action="store_true",
        help="enable ACPKI")
    parser.add_argument("description", metavar="DESCRIPTION",
                        help="JSON system description file")
    args = parser.parse_args()
    description = jsondesc.load(args.description)

    if description["target"] != "kasli_soc":
        raise ValueError("Description is for a different target")

    if description["base"] == "standalone":
        cls = GenericStandalone
    elif description["base"] == "master":
        cls = GenericMaster
    elif description["base"] == "satellite":
        cls = GenericSatellite
    else:
        raise ValueError("Invalid base")

    soc = cls(description, acpki=args.acpki)
    soc.finalize()

    if args.r is not None:
        write_csr_file(soc, args.r)
    if args.c is not None:
        write_rustc_cfg_file(soc, args.c)
    if args.g is not None:
        soc.build(build_dir=args.g)


if __name__ == "__main__":
    main()
