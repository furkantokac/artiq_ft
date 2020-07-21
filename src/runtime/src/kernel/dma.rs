use crate::{
    pl::csr,
    artiq_raise,
    rtio,
};
use alloc::{vec::Vec, string::String, collections::BTreeMap, str};
use cslice::CSlice;
use super::KERNEL_LIBRARY;
use core::mem;
use log::debug;

use libcortex_a9::{
    cache::dcci_slice,
    asm::dsb,
};

const ALIGNMENT: usize = 16 * 8;
const DMA_BUFFER_SIZE: usize = 16 * 8 * 1024;

struct DmaRecorder {
    active:   bool,
    data_len: usize,
    buffer:   [u8; DMA_BUFFER_SIZE],
}

static mut DMA_RECORDER: DmaRecorder = DmaRecorder {
    active:   false,
    data_len: 0,
    buffer:   [0; DMA_BUFFER_SIZE],
};

#[derive(Debug)]
struct Entry {
    trace: Vec<u8>,
    padding_len: usize,
    duration: u64
}

#[derive(Debug)]
pub struct Manager {
    entries: BTreeMap<String, Entry>,
    recording_name: String,
    recording_trace: Vec<u8>
}

// Copied from https://github.com/m-labs/artiq/blob/master/artiq/firmware/runtime/rtio_dma.rs
// basically without modification except removing some warnings.
impl Manager {
    pub fn new() -> Manager {
        Manager {
            entries: BTreeMap::new(),
            recording_name: String::new(),
            recording_trace: Vec::new(),
        }
    }

    pub fn record_start(&mut self, name: &str) {
        self.recording_name = String::from(name);
        self.recording_trace = Vec::new();

        // or we could needlessly OOM replacing a large trace
        self.entries.remove(name);
    }

    pub fn record_append(&mut self, data: &[u8]) {
        self.recording_trace.extend_from_slice(data);
    }

    pub fn record_stop(&mut self, duration: u64) {
        let mut trace = Vec::new();
        mem::swap(&mut self.recording_trace, &mut trace);
        trace.push(0);
        let data_len = trace.len();

        // Realign.
        trace.reserve(ALIGNMENT - 1);
        let padding = ALIGNMENT - trace.as_ptr() as usize % ALIGNMENT;
        let padding = if padding == ALIGNMENT { 0 } else { padding };
        for _ in 0..padding {
            // Vec guarantees that this will not reallocate
            trace.push(0)
        }
        for i in 1..data_len + 1 {
            trace[data_len + padding - i] = trace[data_len - i]
        }

        let mut name = String::new();
        mem::swap(&mut self.recording_name, &mut name);
        self.entries.insert(name, Entry {
            trace, duration,
            padding_len: padding,
        });
    }

    pub fn erase(&mut self, name: &str) {
        self.entries.remove(name);
    }

    pub fn with_trace<F, R>(&self, name: &str, f: F) -> R
            where F: FnOnce(Option<&[u8]>, u64) -> R {
        match self.entries.get(name) {
            Some(entry) => f(Some(&entry.trace[entry.padding_len..]), entry.duration),
            None => f(None, 0)
        }
    }
}


static mut DMA_MANAGER: Option<Manager> = None;

#[repr(C)]
pub struct DmaTrace {
    duration: i64,
    address:  i32,
}

pub fn init_dma() {
    unsafe {
        DMA_MANAGER = Some(Manager::new());
    }
}

fn dma_record_flush() {
    unsafe {
        let manager = DMA_MANAGER.as_mut().unwrap();
        manager.record_append(&DMA_RECORDER.buffer[..DMA_RECORDER.data_len]);
        DMA_RECORDER.data_len = 0;
    }
}

pub extern fn dma_record_start(name: CSlice<u8>) {
    let name = str::from_utf8(name.as_ref()).unwrap();

    unsafe {
        if DMA_RECORDER.active {
            artiq_raise!("DMAError", "DMA is already recording")
        }

        let library = KERNEL_LIBRARY.as_mut().unwrap();
        library.rebind(b"rtio_output",
                       dma_record_output as *const ()).unwrap();
        library.rebind(b"rtio_output_wide",
                       dma_record_output_wide as *const ()).unwrap();

        DMA_RECORDER.active = true;
        let manager = DMA_MANAGER.as_mut().unwrap();
        manager.record_start(name);
    }
}

pub extern fn dma_record_stop(duration: i64) {
    unsafe {
        dma_record_flush();

        if !DMA_RECORDER.active {
            artiq_raise!("DMAError", "DMA is not recording")
        }

        let library = KERNEL_LIBRARY.as_mut().unwrap();
        library.rebind(b"rtio_output",
                       rtio::output as *const ()).unwrap();
        library.rebind(b"rtio_output_wide",
                       rtio::output_wide as *const ()).unwrap();

        DMA_RECORDER.active = false;
        let manager = DMA_MANAGER.as_mut().unwrap();
        manager.record_stop(duration as u64);
    }
}

#[inline(always)]
unsafe fn dma_record_output_prepare(timestamp: i64, target: i32,
                                    words: usize) -> &'static mut [u8] {
    // See gateware/rtio/dma.py.
    const HEADER_LENGTH: usize = /*length*/1 + /*channel*/3 + /*timestamp*/8 + /*address*/1;
    let length = HEADER_LENGTH + /*data*/words * 4;

    if DMA_RECORDER.buffer.len() - DMA_RECORDER.data_len < length {
        dma_record_flush()
    }

    let record = &mut DMA_RECORDER.buffer[DMA_RECORDER.data_len..
                                          DMA_RECORDER.data_len + length];
    DMA_RECORDER.data_len += length;

    let (header, data) = record.split_at_mut(HEADER_LENGTH);

    header.copy_from_slice(&[
        (length    >>  0) as u8,
        (target    >>  8) as u8,
        (target    >>  16) as u8,
        (target    >>  24) as u8,
        (timestamp >>  0) as u8,
        (timestamp >>  8) as u8,
        (timestamp >> 16) as u8,
        (timestamp >> 24) as u8,
        (timestamp >> 32) as u8,
        (timestamp >> 40) as u8,
        (timestamp >> 48) as u8,
        (timestamp >> 56) as u8,
        (target    >>  0) as u8,
    ]);

    data
}

pub extern fn dma_record_output(target: i32, word: i32) {
    unsafe {
        let timestamp = csr::rtio::now_read() as i64;
        let data = dma_record_output_prepare(timestamp, target, 1);
        data.copy_from_slice(&[
            (word >>  0) as u8,
            (word >>  8) as u8,
            (word >> 16) as u8,
            (word >> 24) as u8,
        ]);
    }
}

pub extern fn dma_record_output_wide(target: i32, words: CSlice<i32>) {
    assert!(words.len() <= 16); // enforce the hardware limit

    unsafe {
        let timestamp = csr::rtio::now_read() as i64;
        let mut data = dma_record_output_prepare(timestamp, target, words.len());
        for word in words.as_ref().iter() {
            data[..4].copy_from_slice(&[
                (word >>  0) as u8,
                (word >>  8) as u8,
                (word >> 16) as u8,
                (word >> 24) as u8,
            ]);
            data = &mut data[4..];
        }
    }
}

pub extern fn dma_erase(name: CSlice<u8>) {
    let name = str::from_utf8(name.as_ref()).unwrap();

    let manager = unsafe {
        DMA_MANAGER.as_mut().unwrap()
    };
    manager.erase(name);
}

pub extern fn dma_retrieve(name: CSlice<u8>) -> DmaTrace {
    let name = str::from_utf8(name.as_ref()).unwrap();

    let manager = unsafe {
        DMA_MANAGER.as_mut().unwrap()
    };
    let (trace, duration) = manager.with_trace(name, |trace, duration| (trace.map(|v| {
        dcci_slice(v);
        dsb();
        v.as_ptr()
    }), duration));
    match trace {
        Some(ptr) => Ok(DmaTrace {
            address: ptr as i32,
            duration: duration as i64,
        }),
        None => Err(())
    }.unwrap_or_else(|_| {
        artiq_raise!("DMAError", "DMA trace not found");
    })
}

pub extern fn dma_playback(timestamp: i64, ptr: i32) {
    assert!(ptr % ALIGNMENT as i32 == 0);

    debug!("DMA Playback");
    unsafe {
        csr::rtio_dma::base_address_write(ptr as u32);
        csr::rtio_dma::time_offset_write(timestamp as u64);

        csr::cri_con::selected_write(1);
        csr::rtio_dma::enable_write(1);
        while csr::rtio_dma::enable_read() != 0 {}
        csr::cri_con::selected_write(0);

        let error = csr::rtio_dma::error_read();
        if error != 0 {
            let timestamp = csr::rtio_dma::error_timestamp_read();
            let channel = csr::rtio_dma::error_channel_read();
            csr::rtio_dma::error_write(1);
            if error & 1 != 0 {
                artiq_raise!("RTIOUnderflow",
                    "RTIO underflow at {0} mu, channel {1}",
                    timestamp as i64, channel as i64, 0);
            }
            if error & 2 != 0 {
                artiq_raise!("RTIODestinationUnreachable",
                    "RTIO destination unreachable, output, at {0} mu, channel {1}",
                    timestamp as i64, channel as i64, 0);
            }
        }
    }
}

