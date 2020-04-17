use alloc::{vec, vec::Vec};

use libcortex_a9::{mutex::Mutex, sync_channel::{self, sync_channel}};
use libboard_zynq::println;
use libsupport_zynq::boot::Core1;

use dyld;


#[derive(Debug)]
pub enum Message {
    LoadRequest,
    LoadReply,
}

static CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<Message>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<Message>>> = Mutex::new(None);

pub struct Control {
    core1: Core1<Vec<u32>>,
    pub tx: sync_channel::Sender<Message>,
    pub rx: sync_channel::Receiver<Message>,
}

impl Control {
    pub fn start(stack_size: usize) -> Self {
        let stack = vec![0; stack_size / 4];
        let core1 = Core1::start(stack);

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

#[no_mangle]
pub fn main_core1() {
    println!("Core1 started");

    let mut core1_tx = None;
    while core1_tx.is_none() {
        core1_tx = CHANNEL_1TO0.lock().take();
    }
    let mut core1_tx = core1_tx.unwrap();

    let mut core1_rx = None;
    while core1_rx.is_none() {
        core1_rx = CHANNEL_0TO1.lock().take();
    }
    let core1_rx = core1_rx.unwrap();

    for message in core1_rx {
        println!("core1 received: {:?}", message);
        match *message {
            Message::LoadRequest => core1_tx.send(Message::LoadReply),
            _ => println!("Core1 received unexpected message: {:?}", message),
        }
    }
}
