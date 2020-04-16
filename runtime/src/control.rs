use alloc::{vec, vec::Vec};
use libcortex_a9::{mutex::Mutex, sync_channel::{self, sync_channel}};
use libsupport_zynq::boot::Core1;

pub static CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<usize>>> = Mutex::new(None);
pub static CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<usize>>> = Mutex::new(None);

/// Interface for core 0 to control core 1 start and reset
pub struct KernelControl {
    core1: Core1<Vec<u32>>,
    pub tx: sync_channel::Sender<usize>,
    pub rx: sync_channel::Receiver<usize>,
}

impl KernelControl {
    pub fn start(stack_size: usize) -> Self {
        let stack = vec![0; stack_size / 4];
        let core1 = Core1::start(stack);

        let (core0_tx, core1_rx) = sync_channel(4);
        let (core1_tx, core0_rx) = sync_channel(4);
        *CHANNEL_0TO1.lock() = Some(core1_rx);
        *CHANNEL_1TO0.lock() = Some(core1_tx);

        KernelControl {
            core1,
            tx: core0_tx,
            rx: core0_rx,
        }
    }

    pub fn reset(self) {
        *CHANNEL_0TO1.lock() = None;
        *CHANNEL_1TO0.lock() = None;

        self.core1.reset();
    }
}
