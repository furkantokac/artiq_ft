use libcortex_a9::sync_channel::{self, sync_channel};
use libsupport_zynq::boot::Core1;

use super::{CHANNEL_0TO1, CHANNEL_1TO0, Message};

pub struct Control {
    core1: Core1,
    pub tx: sync_channel::Sender<Message>,
    pub rx: sync_channel::Receiver<Message>,
}

impl Control {
    pub fn start() -> Self {
        let core1 = Core1::start(true);

        let (core0_tx, core1_rx) = sync_channel(4);
        let (core1_tx, core0_rx) = sync_channel(4);
        *CHANNEL_0TO1.lock() = Some(core1_rx);
        *CHANNEL_1TO0.lock() = Some(core1_tx);

        Control {
            core1,
            tx: core0_tx,
            rx: core0_rx,
        }
    }

    pub fn restart(&mut self) {
        *CHANNEL_0TO1.lock() = None;
        *CHANNEL_1TO0.lock() = None;

        self.core1.restart();

        let (core0_tx, core1_rx) = sync_channel(4);
        let (core1_tx, core0_rx) = sync_channel(4);
        *CHANNEL_0TO1.lock() = Some(core1_rx);
        *CHANNEL_1TO0.lock() = Some(core1_tx);
        self.tx = core0_tx;
        self.rx = core0_rx;
    }
}
