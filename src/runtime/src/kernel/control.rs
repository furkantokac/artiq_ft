use libcortex_a9::sync_channel::{self, sync_channel};
use libsupport_zynq::boot::Core1;

use super::{CHANNEL_0TO1, CHANNEL_1TO0, Message};

pub struct Control {
    pub tx: sync_channel::Sender<Message>,
    pub rx: sync_channel::Receiver<Message>,
}

impl Control {
    pub fn start() -> Self {
        Core1::start(true);

        let (core0_tx, core1_rx) = sync_channel(4);
        let (core1_tx, core0_rx) = sync_channel(4);
        *CHANNEL_0TO1.lock() = Some(core1_rx);
        *CHANNEL_1TO0.lock() = Some(core1_tx);

        Control {
            tx: core0_tx,
            rx: core0_rx,
        }
    }
}
