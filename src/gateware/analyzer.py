from migen import *

from misoc.interconnect.csr import *
from misoc.interconnect import stream
from migen_axi.interconnect import axi

from artiq.gateware.rtio.analyzer import message_len, MessageEncoder


def convert_endianness(signal):
    assert len(signal) % 8 == 0
    nbytes = len(signal)//8
    signal_bytes = []
    for i in range(nbytes):
        signal_bytes.append(signal[8*i:8*(i+1)])
    return Cat(*reversed(signal_bytes))


class AXIDMAWriter(Module, AutoCSR):
    def __init__(self, membus, max_outstanding_requests):
        aw = len(membus.aw.addr)
        dw = len(membus.w.data)
        assert message_len % dw == 0
        burst_length = message_len//dw
        alignment_bits = log2_int(message_len//8)

        self.reset = CSR()  # only apply when shut down
        # All numbers in bytes
        self.base_address = CSRStorage(aw, alignment_bits=alignment_bits)
        self.last_address = CSRStorage(aw, alignment_bits=alignment_bits)
        self.byte_count = CSRStatus(32)  # only read when shut down

        self.make_request = Signal()
        self.sink = stream.Endpoint([("data", dw)])

        # # #

        outstanding_requests = Signal(max=max_outstanding_requests+1)
        current_address = Signal(aw - alignment_bits)
        self.comb += [
            membus.aw.addr.eq(Cat(C(0, alignment_bits), current_address)),
            membus.aw.id.eq(0),                   # Same ID for all transactions to forbid reordering.
            membus.aw.burst.eq(axi.Burst.incr.value),
            membus.aw.len.eq(burst_length-1),     # Number of transfers in burst (0->1 transfer, 1->2 transfers...).
            membus.aw.size.eq(log2_int(dw//8)),   # Width of burst: 3 = 8 bytes = 64 bits.
            membus.aw.cache.eq(0xf),
            membus.aw.valid.eq(outstanding_requests != 0),
        ]
        self.sync += [
            outstanding_requests.eq(outstanding_requests + self.make_request - (membus.aw.valid & membus.aw.ready)),

            If(self.reset.re,
                current_address.eq(self.base_address.storage)),
            If(membus.aw.valid & membus.aw.ready,
                If(current_address == self.last_address.storage,
                    current_address.eq(self.base_address.storage)
                ).Else(
                    current_address.eq(current_address + 1)
                )
            )
        ]

        self.comb += [
            membus.w.id.eq(0),
            membus.w.valid.eq(self.sink.stb),
            self.sink.ack.eq(membus.w.ready),
            membus.w.data.eq(convert_endianness(self.sink.data)),
            membus.w.strb.eq(2**(dw//8)-1),
        ]
        beat_count = Signal(max=burst_length)
        self.sync += [
            If(membus.w.valid & membus.w.ready,
                membus.w.last.eq(0),
                If(membus.w.last,
                    beat_count.eq(0)
                ).Else(
                    If(beat_count == burst_length-2, membus.w.last.eq(1)),
                    beat_count.eq(beat_count + 1)
                )
            )
        ]

        message_count = Signal(32 - log2_int(message_len//8))
        self.comb += self.byte_count.status.eq(
            message_count << log2_int(message_len//8))
        self.sync += [
            If(self.reset.re, message_count.eq(0)),
            If(membus.w.valid & membus.w.ready & membus.w.last, message_count.eq(message_count + 1))
        ]

        self.comb += membus.b.ready.eq(1)


class Analyzer(Module, AutoCSR):
    def __init__(self, tsc, cri, membus, fifo_depth=128):
        # shutdown procedure: set enable to 0, wait until busy=0
        self.enable = CSRStorage()
        self.busy = CSRStatus()

        self.submodules.message_encoder = MessageEncoder(
            tsc, cri, self.enable.storage)
        self.submodules.fifo = stream.SyncFIFO(
            [("data", message_len)], fifo_depth, True)
        self.submodules.converter = stream.Converter(
            message_len, len(membus.w.data), reverse=True)
        self.submodules.dma = AXIDMAWriter(membus, max_outstanding_requests=fifo_depth)

        enable_r = Signal()
        self.sync += [
            enable_r.eq(self.enable.storage),
            If(self.enable.storage & ~enable_r,
                self.busy.status.eq(1)),
            If(self.dma.sink.stb & self.dma.sink.ack & self.dma.sink.eop,
                self.busy.status.eq(0))
        ]

        self.comb += [
            self.message_encoder.source.connect(self.fifo.sink),
            self.fifo.source.connect(self.converter.sink),
            self.converter.source.connect(self.dma.sink),
            self.dma.make_request.eq(self.fifo.sink.stb & self.fifo.sink.ack)
        ]
