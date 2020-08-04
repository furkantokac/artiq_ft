use libcortex_a9::sync_channel::{Sender, Receiver};
use libsupport_zynq::boot::Core1;

use super::{CHANNEL_0TO1, CHANNEL_1TO0, Message};

pub struct Control {
    pub tx: Sender<'static, Message>,
    pub rx: Receiver<'static, Message>,
}

fn get_channels() -> (Sender<'static, Message>, Receiver<'static, Message>) {
    let mut core0_tx = None;
    while core0_tx.is_none() {
        core0_tx = CHANNEL_0TO1.lock().take();
    }
    let core0_tx = core0_tx.unwrap();

    let mut core0_rx = None;
    while core0_rx.is_none() {
        core0_rx = CHANNEL_1TO0.lock().take();
    }
    let core0_rx = core0_rx.unwrap();

    (core0_tx, core0_rx)
}

impl Control {
    pub fn start() -> Self {
        Core1::start(true);
        let (core0_tx, core0_rx) = get_channels();

        Control {
            tx: core0_tx,
            rx: core0_rx,
        }
    }
}

