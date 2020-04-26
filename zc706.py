#!/usr/bin/env python

import argparse
import os

from migen import *

from migen_axi.integration.soc_core import SoCCore
from migen_axi.platforms import zc706
from misoc.integration import cpu_interface

from artiq.gateware import rtio
from artiq.gateware.rtio.phy import ttl_simple


class ZC706(SoCCore):
    def __init__(self):
        platform = zc706.Platform()
        platform.toolchain.bitstream_commands.extend([
            "set_property BITSTREAM.GENERAL.COMPRESS True [current_design]",
        ])
        SoCCore.__init__(self, platform=platform, ident="RTIO_ZC706")

        platform.add_platform_command("create_clock -name clk_fpga_0 -period 8 [get_pins \"PS7/FCLKCLK[0]\"]")
        platform.add_platform_command("set_input_jitter clk_fpga_0 0.24")
        self.clock_domains.cd_rtio = ClockDomain()
        self.comb += [
            self.cd_rtio.clk.eq(self.ps7.cd_sys.clk),
            self.cd_rtio.rst.eq(self.ps7.cd_sys.rst)
        ]

        rtio_channels = []
        for i in range(4):
            pad = platform.request("user_led", i)
            phy = ttl_simple.Output(pad)
            self.submodules += phy
            rtio_channels.append(rtio.Channel.from_phy(phy))
        self.add_rtio(rtio_channels)

    def add_rtio(self, rtio_channels):
        self.submodules.rtio_tsc = rtio.TSC("async", glbl_fine_ts_width=3)
        self.submodules.rtio_core = rtio.Core(self.rtio_tsc, rtio_channels)
        self.csr_devices.append("rtio_core")
        self.submodules.rtio = rtio.KernelInitiator(self.rtio_tsc)
        self.csr_devices.append("rtio")

        self.comb += self.rtio.cri.connect(self.rtio_core.cri)

        self.submodules.rtio_moninj = rtio.MonInj(rtio_channels)
        self.csr_devices.append("rtio_moninj")


def write_csr_file(soc, filename):
    with open(filename, "w") as f:
        f.write(cpu_interface.get_csr_rust(
            soc.get_csr_regions(), soc.get_csr_groups(), soc.get_constants()))


def main():
    parser = argparse.ArgumentParser(
        description="ARTIQ port to the ZC706 Zynq development kit")
    parser.add_argument("action", metavar="ACTION", nargs="*",
                        default="gateware rustif".split(),
                        help="actions to perform, default: %(default)s")
    args = parser.parse_args()

    soc = ZC706()
    soc.finalize()

    for action in args.action:
        if action == "gateware":
            soc.build()
        elif action == "rustif":
            write_csr_file(soc, os.path.join("runtime", "src", "pl.rs"))
        else:
            raise ValueError("invalid action", action)


if __name__ == "__main__":
    main()
