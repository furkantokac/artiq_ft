from migen import *
from migen.genlib.fsm import FSM
from misoc.interconnect.csr import *
from misoc.interconnect import stream
from migen_axi.interconnect import axi

from artiq.gateware.rtio.dma import RawSlicer, RecordConverter, RecordSlicer, TimeOffset, CRIMaster


AXI_BURST_LEN = 16


class AXIReader(Module):
    def __init__(self, membus):
        aw = len(membus.ar.addr)
        dw = len(membus.r.data)
        alignment_bits = log2_int(AXI_BURST_LEN*dw//8)
        self.sink = stream.Endpoint([("address", aw - alignment_bits)])
        self.source = stream.Endpoint([("data", dw)])

        # # #

        ar = membus.ar
        r = membus.r

        eop_pending = Signal()
        self.sync += [
            If(self.sink.stb & self.sink.ack & self.sink.eop, eop_pending.eq(1)),
            If(self.source.stb & self.source.ack & self.source.eop, eop_pending.eq(0)),
        ]

        self.comb += [
            ar.addr.eq(Cat(C(0, alignment_bits), self.sink.address)),
            ar.id.eq(0),                   # Same ID for all transactions to forbid reordering.
            ar.burst.eq(axi.Burst.incr.value),
            ar.len.eq(AXI_BURST_LEN-1),    # Number of transfers in burst (0->1 transfer, 1->2 transfers...).
            ar.size.eq(log2_int(dw//8)),   # Width of burst: 3 = 8 bytes = 64 bits.
            ar.cache.eq(0xf),
            ar.valid.eq(self.sink.stb & ~eop_pending),
            self.sink.ack.eq(ar.ready & ~eop_pending)
        ]

        # UG585: "Large slave interface read acceptance capability in the range of 14 to 70 commands"
        inflight_cnt = Signal(max=128)
        self.sync += inflight_cnt.eq(inflight_cnt + (ar.valid & ar.ready) - (r.valid & r.ready))

        self.comb += [
            self.source.stb.eq(r.valid),
            r.ready.eq(self.source.ack),
            self.source.data.eq(r.data),
            self.source.eop.eq(eop_pending & r.last & (inflight_cnt == 0))
        ]


class DMAReader(Module, AutoCSR):
    def __init__(self, membus, enable):
        aw = len(membus.ar.addr)
        alignment_bits = log2_int(AXI_BURST_LEN*len(membus.r.data)//8)

        self.submodules.wb_reader = AXIReader(membus)
        self.source = self.wb_reader.source

        # All numbers in bytes
        self.base_address = CSRStorage(aw, alignment_bits=alignment_bits)

        # # #

        enable_r = Signal()
        address = self.wb_reader.sink
        assert len(address.address) == len(self.base_address.storage)
        self.sync += [
            enable_r.eq(enable),
            If(enable & ~enable_r,
                address.address.eq(self.base_address.storage),
                address.eop.eq(0),
                address.stb.eq(1),
            ),
            If(address.stb & address.ack,
                If(address.eop,
                    address.stb.eq(0)
                ).Else(
                    address.address.eq(address.address + 1),
                    If(~enable, address.eop.eq(1))
                )
            )
        ]


class DMA(Module):
    def __init__(self, membus):
        self.enable = CSR()

        flow_enable = Signal()
        self.submodules.dma = DMAReader(membus, flow_enable)
        self.submodules.slicer = RecordSlicer(len(membus.r.data))
        self.submodules.time_offset = TimeOffset()
        self.submodules.cri_master = CRIMaster()
        self.cri = self.cri_master.cri

        self.comb += [
            self.dma.source.connect(self.slicer.sink),
            self.slicer.source.connect(self.time_offset.sink),
            self.time_offset.source.connect(self.cri_master.sink)
        ]

        fsm = FSM(reset_state="IDLE")
        self.submodules += fsm

        fsm.act("IDLE",
            If(self.enable.re, NextState("FLOWING"))
        )
        fsm.act("FLOWING",
            self.enable.w.eq(1),
            flow_enable.eq(1),
            If(self.slicer.end_marker_found,
                NextState("FLUSH")
            )
        )
        fsm.act("FLUSH",
            self.enable.w.eq(1),
            self.slicer.flush.eq(1),
            NextState("WAIT_EOP")
        )
        fsm.act("WAIT_EOP",
            self.enable.w.eq(1),
            If(self.cri_master.sink.stb & self.cri_master.sink.ack & self.cri_master.sink.eop,
                NextState("WAIT_CRI_MASTER")
            )
        )
        fsm.act("WAIT_CRI_MASTER",
            self.enable.w.eq(1),
            If(~self.cri_master.busy, NextState("IDLE"))
        )

    def get_csrs(self):
        return ([self.enable] +
                self.dma.get_csrs() + self.time_offset.get_csrs() +
                self.cri_master.get_csrs())
