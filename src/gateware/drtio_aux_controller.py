"""Auxiliary controller, common to satellite and master"""

from artiq.gateware.drtio.aux_controller import (max_packet, aux_buffer_count,
    Transmitter, Receiver)
from migen.fhdl.simplify import FullMemoryWE
from misoc.interconnect.csr import *
from migen_axi.interconnect.sram import SRAM
from migen_axi.interconnect import axi


class _DRTIOAuxControllerBase(Module):
    def __init__(self, link_layer):
        self.bus = axi.Interface()
        self.submodules.transmitter = Transmitter(link_layer, len(self.bus.w.data))
        self.submodules.receiver = Receiver(link_layer, len(self.bus.w.data))

    def get_csrs(self):
        return self.transmitter.get_csrs() + self.receiver.get_csrs()


# TODO: FullMemoryWE should be applied by migen.build
@FullMemoryWE()
class DRTIOAuxControllerAxi(_DRTIOAuxControllerBase):
    def __init__(self, link_layer):
        _DRTIOAuxControllerBase.__init__(self, link_layer)

        tx_sdram_if = SRAM(self.transmitter.mem, read_only=False)
        rx_sdram_if = SRAM(self.receiver.mem, read_only=True)
        aw_decoder = axi.AddressDecoder(self.bus.aw,
            [(lambda a: a[log2_int(max_packet*aux_buffer_count)] == 0, tx_sdram_if.bus.aw),
             (lambda a: a[log2_int(max_packet*aux_buffer_count)] == 1, rx_sdram_if.bus.aw)],
            register=True)
        ar_decoder = axi.AddressDecoder(self.bus.ar,
            [(lambda a: a[log2_int(max_packet*aux_buffer_count)] == 0, tx_sdram_if.bus.ar),
             (lambda a: a[log2_int(max_packet*aux_buffer_count)] == 1, rx_sdram_if.bus.ar)],
            register=True)
        # unlike wb, axi address decoder only connects ar/aw lanes,
        # the rest must also be connected!
        # not quite unlike an address decoder itself.

        # connect bus.b with tx.b
        self.comb += [tx_sdram_if.bus.b.ready.eq(self.bus.b.ready),
                      self.bus.b.id.eq(tx_sdram_if.bus.b.id),
                      self.bus.b.resp.eq(tx_sdram_if.bus.b.resp),
                      self.bus.b.valid.eq(tx_sdram_if.bus.b.valid)]
        # connect bus.w with tx.w
        # no worries about w.valid and slave sel here, only tx will be written to
        self.comb += [tx_sdram_if.bus.w.id.eq(self.bus.w.id),
                      tx_sdram_if.bus.w.data.eq(self.bus.w.data),
                      tx_sdram_if.bus.w.strb.eq(self.bus.w.strb),
                      tx_sdram_if.bus.w.last.eq(self.bus.w.last),
                      tx_sdram_if.bus.w.valid.eq(self.bus.w.valid),
                      self.bus.w.ready.eq(tx_sdram_if.bus.w.ready)]
        # connect bus.r with rx.r and tx.r w/o data
        self.comb += [self.bus.r.id.eq(rx_sdram_if.bus.r.id | tx_sdram_if.bus.r.id),
                #self.bus.r.data.eq(rx_sdram_if.bus.r.data | tx_sdram_if.bus.r.data),
                self.bus.r.resp.eq(rx_sdram_if.bus.r.resp | tx_sdram_if.bus.r.resp),
                self.bus.r.last.eq(rx_sdram_if.bus.r.last | tx_sdram_if.bus.r.last),
                self.bus.r.valid.eq(rx_sdram_if.bus.r.valid | tx_sdram_if.bus.r.valid),
                rx_sdram_if.bus.r.ready.eq(self.bus.r.ready),
                tx_sdram_if.bus.r.ready.eq(self.bus.r.ready)]
        # connect read data after being masked
        masked = [Replicate(rx_sdram_if.bus.r.valid,
                            len(self.bus.r.data)
                            ) & rx_sdram_if.bus.r.data,
                 Replicate(tx_sdram_if.bus.r.valid,
                            len(self.bus.r.data)
                            ) & tx_sdram_if.bus.r.data]
        self.comb += self.bus.r.data.eq(reduce(or_, masked))

        self.submodules += tx_sdram_if, rx_sdram_if, aw_decoder, ar_decoder


@FullMemoryWE()
class DRTIOAuxControllerBare(_DRTIOAuxControllerBase):
    # Barebones version of the AuxController. No SRAM, no decoders.
    # add memories manually from tx and rx in target code.
    def get_tx_port(self):
        return self.transmitter.mem.get_port(write_capable=True)

    def get_rx_port(self):
        return self.receiver.mem.get_port(write_capable=False)

    def get_mem_size(self):
        return max_packet*aux_buffer_count
