import unittest
import random
import itertools

from migen import *
from migen_axi.interconnect import axi

from artiq.coredevice.exceptions import RTIOUnderflow, RTIODestinationUnreachable
from artiq.gateware import rtio
from artiq.gateware.rtio import cri
from artiq.gateware.rtio.phy import ttl_simple

import endianness
import dma


class AXIMemorySim:
    def __init__(self, bus, data, max_queue=12):
        self.bus = bus
        self.data = data
        self.max_queue = max_queue
        self.align = len(bus.r.data)//8
        self.queue = []

    @passive
    def ar(self):
        while True:
            if len(self.queue) < self.max_queue:
                request = yield from self.bus.read_ar()
                self.queue.append(request)
            else:
                yield

    @passive
    def r(self):
        while True:
            if self.queue:
                request = self.queue.pop()
                if request.burst:
                    request_len = request.len + 1
                else:
                    request_len = 1
                for i in range(request_len):
                    if request.addr % self.align:
                        raise ValueError
                    addr = request.addr//self.align + i
                    if addr < len(self.data):
                        data = self.data[addr]
                    else:
                        data = 0
                    data = endianness.convert_value(data, len(self.bus.r.data))
                    yield from self.bus.write_r(request.id, data, last=i == request_len-1)
            else:
                yield


def encode_n(n, min_length, max_length):
    r = []
    while n:
        r.append(n & 0xff)
        n >>= 8
    r += [0]*(min_length - len(r))
    if len(r) > max_length:
        raise ValueError
    return r


def encode_record(channel, timestamp, address, data):
    r = []
    r += encode_n(channel, 3, 3)
    r += encode_n(timestamp, 8, 8)
    r += encode_n(address, 1, 1)
    r += encode_n(data, 1, 64)
    return encode_n(len(r)+1, 1, 1) + r


def pack(x, size):
    r = []
    for i in range((len(x)+size-1)//size):
        n = 0
        for j in range(i*size, (i+1)*size):
            n <<= 8
            try:
                n |= x[j]
            except IndexError:
                pass
        r.append(n)
    return r


def encode_sequence(writes, ws):
    sequence = [b for write in writes for b in encode_record(*write)]
    sequence.append(0)
    return pack(sequence, ws)


def do_dma(dut, address):
    yield from dut.dma.base_address.write(address)
    yield from dut.enable.write(1)
    yield
    while ((yield from dut.enable.read())):
        yield
    error = yield from dut.cri_master.error.read()
    if error & 1:
        raise RTIOUnderflow
    if error & 2:
        raise RTIODestinationUnreachable


test_writes1 = [
    (0x01, 0x23, 0x12, 0x33),
    (0x901, 0x902, 0x11, 0xeeeeeeeeeeeeeefffffffffffffffffffffffffffffff28888177772736646717738388488),
    (0x81, 0x288, 0x88, 0x8888)
]


test_writes2 = [
    (0x10, 0x10000, 0x20, 0x77),
    (0x11, 0x10001, 0x22, 0x7777),
    (0x12, 0x10002, 0x30, 0x777777),
    (0x13, 0x10003, 0x40, 0x77777788),
    (0x14, 0x10004, 0x50, 0x7777778899),
]


prng = random.Random(0)


class TB(Module):
    def __init__(self, ws):
        sequence1 = encode_sequence(test_writes1, ws)
        sequence2 = encode_sequence(test_writes2, ws)
        offset = 512//ws
        assert len(sequence1) < offset
        sequence = (
            sequence1 +
            [prng.randrange(2**(ws*8)) for _ in range(offset-len(sequence1))] +
            sequence2)

        bus = axi.Interface(ws*8)
        self.memory = AXIMemorySim(bus, sequence)
        self.submodules.dut = dma.DMA(bus)


test_writes_full_stack = [
    (0, 32, 0, 1),
    (1, 40, 0, 1),
    (0, 48, 0, 0),
    (1, 50, 0, 0),
]


class FullStackTB(Module):
    def __init__(self, ws):
        self.ttl0 = Signal()
        self.ttl1 = Signal()

        self.submodules.phy0 = ttl_simple.Output(self.ttl0)
        self.submodules.phy1 = ttl_simple.Output(self.ttl1)

        rtio_channels = [
            rtio.Channel.from_phy(self.phy0),
            rtio.Channel.from_phy(self.phy1)
        ]

        sequence = encode_sequence(test_writes_full_stack, ws)

        bus = axi.Interface(ws*8)
        self.memory = AXIMemorySim(bus, sequence)
        self.submodules.dut = dma.DMA(bus)
        self.submodules.tsc = rtio.TSC("async")
        self.submodules.rtio = rtio.Core(self.tsc, rtio_channels)
        self.comb += self.dut.cri.connect(self.rtio.cri)


class TestDMA(unittest.TestCase):
    def test_dma_noerror(self):
        tb = TB(8)

        def do_writes():
            yield from do_dma(tb.dut, 0)
            yield from do_dma(tb.dut, 512)

        received = []
        @passive
        def rtio_sim():
            dut_cri = tb.dut.cri
            while True:
                cmd = yield dut_cri.cmd
                if cmd == cri.commands["nop"]:
                    pass
                elif cmd == cri.commands["write"]:
                    channel = yield dut_cri.chan_sel
                    timestamp = yield dut_cri.o_timestamp
                    address = yield dut_cri.o_address
                    data = yield dut_cri.o_data
                    received.append((channel, timestamp, address, data))

                    yield dut_cri.o_status.eq(1)
                    for i in range(prng.randrange(10)):
                        yield
                    yield dut_cri.o_status.eq(0)
                else:
                    self.fail("unexpected RTIO command")
                yield

        run_simulation(tb, [do_writes(), rtio_sim(), tb.memory.ar(), tb.memory.r()])
        self.assertEqual(received, test_writes1 + test_writes2)

    def test_full_stack(self):
        tb = FullStackTB(8)

        ttl_changes = []
        @passive
        def monitor():
            old_ttl_states = [0, 0]
            for time in itertools.count():
                ttl_states = [
                    (yield tb.ttl0),
                    (yield tb.ttl1)
                ]
                for i, (old, new) in enumerate(zip(old_ttl_states, ttl_states)):
                    if new != old:
                        ttl_changes.append((time, i))
                old_ttl_states = ttl_states
                yield

        run_simulation(tb, {"sys": [
            do_dma(tb.dut, 0), monitor(),
            (None for _ in range(70)),
            tb.memory.ar(), tb.memory.r()
        ]}, {"sys": 8, "rsys": 8, "rtio": 8, "rio": 8, "rio_phy": 8})

        correct_changes = [(timestamp + 11, channel)
                           for channel, timestamp, _, _ in test_writes_full_stack]
        self.assertEqual(ttl_changes, correct_changes)
