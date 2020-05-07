#!/usr/bin/env python

import argparse

from migen import *

from migen_axi.integration.soc_core import SoCCore
from migen_axi.platforms import zc706
from misoc.integration import cpu_interface

from artiq.gateware import rtio, nist_clock, nist_qc2
from artiq.gateware.rtio.phy import ttl_simple, ttl_serdes_7series, dds, spi2


class ZC706(SoCCore):
    def __init__(self):
        platform = zc706.Platform()
        platform.toolchain.bitstream_commands.extend([
            "set_property BITSTREAM.GENERAL.COMPRESS True [current_design]",
        ])
        SoCCore.__init__(self, platform=platform, ident=self.__class__.__name__)

        platform.add_platform_command("create_clock -name clk_fpga_0 -period 8 [get_pins \"PS7/FCLKCLK[0]\"]")
        platform.add_platform_command("set_input_jitter clk_fpga_0 0.24")
        self.clock_domains.cd_rtio = ClockDomain()
        self.comb += [
            self.cd_rtio.clk.eq(self.ps7.cd_sys.clk),
            self.cd_rtio.rst.eq(self.ps7.cd_sys.rst)
        ]

    def add_rtio(self, rtio_channels):
        self.submodules.rtio_tsc = rtio.TSC("async", glbl_fine_ts_width=3)
        self.submodules.rtio_core = rtio.Core(self.rtio_tsc, rtio_channels)
        self.csr_devices.append("rtio_core")
        self.submodules.rtio = rtio.KernelInitiator(self.rtio_tsc, now64=True)
        self.csr_devices.append("rtio")

        self.comb += self.rtio.cri.connect(self.rtio_core.cri)

        self.submodules.rtio_moninj = rtio.MonInj(rtio_channels)
        self.csr_devices.append("rtio_moninj")


class Simple(ZC706):
    def __init__(self):
        ZC706.__init__(self)

        platform = self.platform

        rtio_channels = []
        for i in range(4):
            pad = platform.request("user_led", i)
            phy = ttl_simple.Output(pad)
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(phy))
        self.add_rtio(rtio_channels)


class NIST_CLOCK(ZC706):
    """
    NIST clock hardware, with old backplane and 11 DDS channels
    """
    def __init__(self):
        ZC706.__init__(self)

        platform = self.platform
        platform.add_extension(nist_clock.fmc_adapter_io)

        rtio_channels = []
        for i in range(16):
            if i % 4 == 3:
                phy = ttl_simple.InOut(platform.request("ttl", i))
                self.submodules += phy
                rtio_channels.append(rtio.Channel.from_phy(phy, ififo_depth=512))
            else:
                phy = ttl_simple.Output(platform.request("ttl", i))
                self.submodules += phy
                rtio_channels.append(rtio.Channel.from_phy(phy))

        for i in range(2):
            phy = ttl_simple.InOut(platform.request("pmt", i))
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(phy, ififo_depth=512))

        phy = ttl_simple.Output(platform.request("user_led", 2))
        self.submodules += phy
        rtio_channels.append(rtio.Channel.from_phy(phy))

        phy = ttl_simple.ClockGen(platform.request("la32_p"))
        self.submodules += phy
        rtio_channels.append(rtio.Channel.from_phy(phy))

        for i in range(3):
            phy = spi2.SPIMaster(self.platform.request("spi", i))
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(
                phy, ififo_depth=128))

        phy = dds.AD9914(platform.request("dds"), 11, onehot=True)
        self.submodules += phy
        rtio_channels.append(rtio.Channel.from_phy(phy, ififo_depth=4))

        self.add_rtio(rtio_channels)


class NIST_QC2(ZC706):
    """
    NIST QC2 hardware, as used in Quantum I and Quantum II, with new backplane
    and 24 DDS channels.  Two backplanes are used.
    """
    def __init__(self):
        ZC706.__init__(self)

        platform = self.platform
        platform.add_extension(nist_qc2.fmc_adapter_io)

        rtio_channels = []
        clock_generators = []

        # All TTL channels are In+Out capable
        for i in range(40):
            phy = ttl_simple.InOut(
                platform.request("ttl", i))
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(phy, ififo_depth=512))

        # CLK0, CLK1 are for clock generators, on backplane SMP connectors
        for i in range(2):
            phy = ttl_simple.ClockGen(
                platform.request("clkout", i))
            self.submodules += phy
            clock_generators.append(rtio.Channel.from_phy(phy))

        phy = ttl_simple.Output(platform.request("user_led", 2))
        self.submodules += phy
        rtio_channels.append(rtio.Channel.from_phy(phy))

        # add clock generators after TTLs
        rtio_channels += clock_generators

        for i in range(4):
            phy = spi2.SPIMaster(self.platform.request("spi", i))
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(
                phy, ififo_depth=128))

        for backplane_offset in range(2):
            phy = dds.AD9914(
                platform.request("dds", backplane_offset), 12, onehot=True)
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(phy, ififo_depth=4))

        self.add_rtio(rtio_channels)


VARIANTS = {cls.__name__.lower(): cls for cls in [Simple, NIST_CLOCK, NIST_QC2]}


def write_csr_file(soc, filename):
    with open(filename, "w") as f:
        f.write(cpu_interface.get_csr_rust(
            soc.get_csr_regions(), soc.get_csr_groups(), soc.get_constants()))


def main():
    parser = argparse.ArgumentParser(
        description="ARTIQ port to the ZC706 Zynq development kit")
    parser.add_argument("-r", default=None,
        help="build Rust interface into the specified file")
    parser.add_argument("-g", default=None,
        help="build gateware into the specified directory")
    parser.add_argument("-V", "--variant", default="simple",
                        help="variant: "
                             "simple/nist_clock/nist_qc2 "
                             "(default: %(default)s)")
    args = parser.parse_args()

    try:
        cls = VARIANTS[args.variant.lower()]
    except KeyError:
        raise SystemExit("Invalid variant (-V/--variant)")

    soc = cls()
    soc.finalize()

    if args.g is not None:
        soc.build(build_dir=args.g)
    if args.r is not None:
        write_csr_file(soc, args.r)


if __name__ == "__main__":
    main()
