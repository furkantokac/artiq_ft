from migen import *


def convert_signal(signal):
    assert len(signal) % 8 == 0
    nbytes = len(signal)//8
    signal_bytes = []
    for i in range(nbytes):
        signal_bytes.append(signal[8*i:8*(i+1)])
    return Cat(*reversed(signal_bytes))


def convert_value(value, size):
    assert size % 8 == 0
    nbytes = size//8
    result = 0
    for i in range(nbytes):
        result <<= 8
        result |= value & 0xff
        value >>= 8
    return result
